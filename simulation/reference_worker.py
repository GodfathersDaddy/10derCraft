"""Optional external reference solver, independent of the SolidCraft Rust runtime.

Reads one versioned JSON job, returns one JSON result. No eval, user scripts or
automatic CAD modifications. Public Exudyn/SciPy APIs only; no vendored solver code.
"""
from __future__ import annotations

import argparse
from contextlib import redirect_stdout
from datetime import datetime, timezone
from hashlib import sha256
from importlib.metadata import version
import json
import math
from pathlib import Path
import platform
import sys
import time

MAX_BYTES = 2_000_000
FIELDS = {
    'length_mm': (0.001, 1e6), 'thickness_mm': (0.001, 1e5),
    'height_mm': (0.001, 1e5), 'young_mpa': (0.001, 1e7),
    'density_kg_m3': (0.001, 1e6), 'force_n': (-1e9, 1e9),
    'deflection_limit_mm': (0.000001, 1e6),
}


def number(value, name, lower, upper):
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f'{name}: a number is required')
    if not math.isfinite(value) or not lower <= value <= upper:
        raise ValueError(f'{name}: outside [{lower}, {upper}]')
    return float(value)


def integer(value, name, lower, upper):
    number(value, name, lower, upper)
    if int(value) != value:
        raise ValueError(f'{name}: integer required')
    return int(value)


def beam_values(p):
    inertia = p['thickness_mm'] * p['height_mm'] ** 3 / 12
    displacement = p['force_n'] * p['length_mm'] ** 3 / (3 * p['young_mpa'] * inertia)
    mass = p['density_kg_m3'] * p['length_mm'] * p['thickness_mm'] * p['height_mm'] * 1e-9
    stiffness = 3000 * p['young_mpa'] * inertia / p['length_mm'] ** 3
    stress = abs(p['force_n']) * p['length_mm'] * p['height_mm'] / (2 * inertia)
    return dict(mass_kg=mass, tip_deflection_mm=displacement,
                tip_stiffness_n_m=stiffness, root_bending_stress_mpa=stress)


def validate(job):
    if not isinstance(job, dict) or job.get('schema') != 'solidcraft.reference-job/1':
        raise ValueError('unsupported job schema')
    if job.get('backend') not in ('scipy_beam_size', 'exudyn_beam_reference'):
        raise ValueError('unsupported backend')
    reference = job.get('reference')
    if not isinstance(reference, dict) or reference.get('model') != 'euler_bernoulli_rectangular_cantilever/v1':
        raise ValueError('unsupported reference model')
    inputs = reference.get('inputs')
    if not isinstance(inputs, dict) or set(inputs) != set(FIELDS):
        raise ValueError('beam inputs must contain exactly the documented fields')
    p = {k: number(inputs[k], k, *limits) for k, limits in FIELDS.items()}
    if not isinstance(job.get('settings'), dict):
        raise ValueError('settings object required')
    return p


def scipy_size(p, settings):
    import scipy
    from scipy.optimize import minimize

    def bounds(key):
        pair = settings.get(key)
        if not isinstance(pair, list) or len(pair) != 2:
            raise ValueError(f'{key}: two bounds required')
        low, high = (number(v, key, 0.001, 1e5) for v in pair)
        if low >= high:
            raise ValueError(f'{key}: increasing bounds required')
        return low, high

    t_bounds = bounds('thickness_bounds_mm')
    h_bounds = bounds('height_bounds_mm')
    # Keep this first optimization inside the reference model's declared regime.
    h_bounds = (h_bounds[0], min(h_bounds[1], p['length_mm']/10))
    if h_bounds[1] <= h_bounds[0]:
        raise ValueError('height bounds leave no slender beam domain (length/height >= 10)')
    effective_limit = min(p['deflection_limit_mm'], .02*p['length_mm'])
    maxiter = integer(settings.get('max_iterations', 100), 'max_iterations', 1, 300)
    # Scale both independent dimensions to [0,1] and objective to the initial mass.
    def candidate(x):
        return dict(p, thickness_mm=float(t_bounds[0]+x[0]*(t_bounds[1]-t_bounds[0])),
                    height_mm=float(h_bounds[0]+x[1]*(h_bounds[1]-h_bounds[0])))

    mass_scale = beam_values(p)['mass_kg']
    def objective(x):
        return beam_values(candidate(x))['mass_kg'] / mass_scale

    def margin(x):
        return 1 - abs(beam_values(candidate(x))['tip_deflection_mm']) / effective_limit

    largest = dict(p, thickness_mm=t_bounds[1], height_mm=h_bounds[1])
    if abs(beam_values(largest)['tip_deflection_mm']) > effective_limit:
        return dict(status='infeasible', message='Even the largest allowed section violates the deflection limit.',
                    backend_version=scipy.__version__, summary=beam_values(largest))

    x0 = [min(1., max(0., (p[k]-b[0])/(b[1]-b[0])))
          for k, b in [('thickness_mm', t_bounds), ('height_mm', h_bounds)]]
    trace = []
    def record(x):
        point = candidate(x)
        trace.append(dict(iteration=len(trace), thickness_mm=point['thickness_mm'],
                          height_mm=point['height_mm'], **beam_values(point)))

    record(x0)
    opt = minimize(objective, x0, method='SLSQP', bounds=[(0., 1.), (0., 1.)],
                   constraints=[{'type':'ineq', 'fun':margin}], callback=record,
                   options={'maxiter':maxiter, 'ftol':1e-10, 'disp':False})
    final = candidate(opt.x)
    summary = dict(beam_values(final), thickness_mm=final['thickness_mm'], height_mm=final['height_mm'])
    feasible = abs(summary['tip_deflection_mm']) <= effective_limit*(1+1e-7)
    return dict(status='success' if opt.success and feasible else 'failed',
                message=str(opt.message), backend_version=scipy.__version__,
                solver='scipy.optimize.minimize/SLSQP', summary=summary,
                iterations=int(opt.nit), function_evaluations=int(opt.nfev), trace=trace,
                constraint_satisfied=feasible,
                applicability_constraints={'length_height_ratio_min':10., 'deflection_length_ratio_max':.02},
                qualification='Local analytic beam sizing only; not a topology optimization or certified global optimum.')


def exudyn_reference(p, settings):
    import exudyn as exu
    from exudyn.itemInterface import (
        NodePointGround, Node1D, Mass1D, MarkerNodeCoordinate,
        CoordinateSpringDamper, LoadCoordinate, SensorNode,
    )

    tip_mass = number(settings.get('tip_mass_kg', 5.), 'tip_mass_kg', 0.001, 1e6)
    zeta = number(settings.get('damping_ratio', 0.02), 'damping_ratio', 0., 1.)
    duration = number(settings.get('duration_s', .2), 'duration_s', 1e-5, 10.)
    steps = integer(settings.get('steps', 4000), 'steps', 10, 20000)
    reference = beam_values(p)
    # A static-stiffness equivalent oscillator with tip mass only. The beam's
    # distributed inertia is deliberately omitted; this is not a FEM/CMS ROM.
    stiffness = reference['tip_stiffness_n_m']
    omega = math.sqrt(stiffness / tip_mass)
    if duration * omega / steps > .05:
        raise ValueError('time step too coarse for this oscillator; increase steps or shorten duration')
    damping = 2*zeta*math.sqrt(stiffness*tip_mass)
    sc = exu.SystemContainer()
    mbs = sc.AddSystem()
    ground = mbs.AddNode(NodePointGround(referenceCoordinates=[0.,0.,0.]))
    mass_node = mbs.AddNode(Node1D(referenceCoordinates=[0.], initialCoordinates=[0.], initialVelocities=[0.]))
    mbs.AddObject(Mass1D(nodeNumber=mass_node, physicsMass=tip_mass))
    fixed = mbs.AddMarker(MarkerNodeCoordinate(nodeNumber=ground, coordinate=0))
    moving = mbs.AddMarker(MarkerNodeCoordinate(nodeNumber=mass_node, coordinate=0))
    mbs.AddObject(CoordinateSpringDamper(markerNumbers=[fixed,moving], stiffness=stiffness, damping=damping))
    mbs.AddLoad(LoadCoordinate(markerNumber=moving, load=p['force_n']))
    sensor = mbs.AddSensor(SensorNode(nodeNumber=mass_node, outputVariableType=exu.OutputVariableType.Coordinates,
                                    storeInternal=True, writeToFile=False))
    mbs.Assemble()
    options = exu.SimulationSettings()
    options.timeIntegration.numberOfSteps = steps
    options.timeIntegration.endTime = duration
    options.timeIntegration.verboseMode = 0
    options.timeIntegration.generalizedAlpha.spectralRadius = 1.
    options.solutionSettings.writeSolutionToFile = False
    options.solutionSettings.sensorsWritePeriod = duration/steps
    mbs.SolveDynamic(options, solverType=exu.DynamicSolverType.GeneralizedAlpha)
    data = mbs.GetSensorStoredData(sensor)
    series = [[float(row[0]), float(row[1])] for row in data]
    if len(series) < 2 or abs(series[-1][0]-duration) > duration*1e-6:
        raise ValueError('missing or incomplete Exudyn sensor output')
    # Independent exact underdamped/critical step response; compare all raw samples.
    equilibrium = p['force_n']/stiffness
    def exact(t):
        if zeta == 1.:
            return equilibrium*(1-math.exp(-omega*t)*(1+omega*t))
        wd = omega*math.sqrt(1-zeta*zeta)
        return equilibrium*(1-math.exp(-zeta*omega*t)*(math.cos(wd*t)+zeta*omega/wd*math.sin(wd*t)))
    error = max(abs(displacement-exact(t)) for t, displacement in series)
    tolerance = max(1e-9, abs(equilibrium)*1e-3)
    summary = dict(static_deflection_mm=reference['tip_deflection_mm'], tip_stiffness_n_m=stiffness,
                   natural_frequency_hz=omega/(2*math.pi), final_displacement_mm=series[-1][1]*1000,
                   maximum_reference_error_mm=error*1000, tip_mass_kg=tip_mass)
    return dict(status='success' if error <= tolerance else 'failed', backend_version=version('exudyn'),
                solver='Exudyn/GeneralizedAlpha', summary=summary,
                message='Compared with the exact oscillator step response.',
                time_series=series, time_series_units=['s','m'], reference_tolerance_m=tolerance,
                qualification='1-DOF tip-mass oscillator with analytic beam stiffness; no distributed beam inertia, no CAD-derived MKS or FEM/CMS ROM.')


def run(job):
    started = time.perf_counter()
    inputs = validate(job)
    # Exudyn can write progress to stdout; keep the result protocol clean.
    with redirect_stdout(sys.stderr):
        payload = scipy_size(inputs, job['settings']) if job['backend']=='scipy_beam_size' else exudyn_reference(inputs, job['settings'])
    return dict(schema='solidcraft.reference-result/1', job=job, **payload,
                provenance=dict(created_utc=datetime.now(timezone.utc).isoformat(),
                                elapsed_s=time.perf_counter()-started, python=platform.python_version(),
                                job_sha256=sha256(json.dumps(job,sort_keys=True,separators=(',',':'),allow_nan=False).encode()).hexdigest(),
                                worker_sha256=sha256(Path(__file__).read_bytes()).hexdigest()))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('job',type=Path)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args()
    job=None
    try:
        if args.job.resolve()==args.out.resolve():
            raise ValueError('input and output must be different files')
        if args.out.exists():
            raise ValueError('output exists; choose a new result filename')
        if args.job.stat().st_size>MAX_BYTES:
            raise ValueError('job exceeds size limit')
        job=json.loads(args.job.read_text(encoding='utf-8-sig'), parse_constant=lambda x: (_ for _ in ()).throw(ValueError('non-finite JSON number')))
        result=run(job)
        text=json.dumps(result,ensure_ascii=False,indent=2,allow_nan=False)
        # Exclusive create: never overwrite previous research results.
        with args.out.open('x',encoding='utf-8') as stream:
            stream.write(text+'\n')
        print(json.dumps({'status':result['status'],'output':str(args.out)},ensure_ascii=False))
        return 0 if result['status']=='success' else 2
    except Exception as exc:
        print(json.dumps({'status':'failed','error':str(exc)},ensure_ascii=False),file=sys.stderr)
        return 1


if __name__=='__main__':
    raise SystemExit(main())
