"""Independent SI reference calculations against the actual local Calcpad process.

No product equation evaluator is reused. Tooth counts are found by enumerating
mountable candidates; power/torque balances and beam equilibrium are checked in SI.
Passing arithmetic does not imply that every requested design is feasible.
"""
import concurrent.futures
import hashlib
import itertools
import json
import math
from pathlib import Path
import re
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'output/kue-comprehensive'
SOURCE = ROOT / 'crates/engine/src/calculations/kue-stage.cpd'
DATA = {
    1: (1e6,22,1000,.995,.990,.975,.34),
    2: (5e5,30,1200,.995,.990,.970,.35),
    3: (3e5,40,1500,.990,.980,.965,.36),
    4: (1e5,60,1800,.990,.980,.960,.38),
}
MODES = {1:(1,1),2:(1,2),3:(2,1),4:(2,2)}
PREFERRED = [.5,.6,.8,1,1.25,1.5,2,2.5,3,4,5,6,8,10,12,16,20,25,32,40,50,63,80,100,200]
SCALES = {
    '': (1,'scalar'), 'rad':(1,'scalar'),
    'm':(1,'length'),'mm':(.001,'length'),
    'm^4':(1,'inertia'),'mm^4':(1e-12,'inertia'),
    'min^(-1)':(1/60,'frequency'),'s^(-1)':(1,'frequency'),'Hz':(1,'frequency'),
    'W':(1,'power'),'kW':(1e3,'power'),'MW':(1e6,'power'),
    't\u00b7m^2\u2215s^3':(1e3,'power'),'kg\u00b7m^2\u2215s^3':(1,'power'),
    'Nm':(1,'torque'),'kNm':(1e3,'torque'),'Nmm':(.001,'torque'),
    'N\u00b7m':(1,'torque'),
    'N':(1,'force'),'kN':(1e3,'force'),'MN':(1e6,'force'),
    'Pa':(1,'stress'),'kPa':(1e3,'stress'),'MPa':(1e6,'stress'),'GPa':(1e9,'stress'),
    'N\u2215mm^2':(1e6,'stress'),
}


def edited(source, name, expression):
    source, count = re.subn(rf'(?m)^{re.escape(name)} = .*$', f'{name} = {expression}', source)
    assert count == 1, (name, count)
    return source


def choose_planet(sun, target, mode):
    # Enumerate both mounting and noninteger-ratio requirements independently.
    candidates = []
    for planet in range(1, 501):
        ring = sun+2*planet
        ratio = 1+ring/sun if mode == 1 else ring/sun
        if (sun+ring) % 3 == 0 and abs(ratio-round(ratio)) > 1e-9:
            candidates.append((abs(ratio-target), planet, ratio))
    error = min(c[0] for c in candidates)
    tied = [c for c in candidates if abs(c[0]-error) < 1e-10]
    # Equally good integer counts are legitimate alternatives.
    return tied


def reference(case, results):
    p,n,ng,e1,e2,ess,iss = DATA[case['variant']]
    mode1,mode2 = MODES[case['layout']]
    n = case.get('rpm',n)
    torque = case.get('torque',1.1*p/(2*math.pi*n/60))
    zs = case.get('sun',30)
    target = case.get('ratio',3.4)
    width_factor = case.get('width',15)
    wanted = {}
    def expected(name, value, dimension='scalar'):
        wanted[name] = (value,dimension)
    for name,value in [('Variante',case['variant']),('Aufbauvariante',case['layout']),('mode_1',mode1),('mode_2',mode2),('K_A',1.1),('R_Ziel',target),('z_s',zs),('z_s2',30),('n_Planeten',3),('alpha_grad',case.get('alpha',20)),('breitenfaktor',width_factor),('Y_F',.3),('K_gamma',1.25),('eta_1',e1),('eta_2',e2),('eta_SS',ess),('i_SS',iss),('offset_1',3-mode1),('offset_2',3-mode2)]:
        expected(name,value)
    for name,value in [('sigma_F_zul',160e6),('sigma_Bolzen_zul',200e6),('tau_Welle_zul',60e6),('E_Stahl',210e9)]: expected(name,value,'stress')
    expected('P_Rotor',p,'power')
    expected('n_Rotor',DATA[case['variant']][1]/60,'frequency')
    expected('n_Generator_Ziel',ng/60,'frequency')
    expected('n_in',n/60,'frequency')
    expected('T_in',torque,'torque')
    expected('delta_Bolzen_zul',.00004,'length')
    expected('theta_Bolzen_zul',.005)
    zp = int(results['z_p']['value'])
    assert zp in [c[1] for c in choose_planet(zs,target,mode1)], ('stage1 selection',zp,choose_planet(zs,target,mode1))
    zr = zs+2*zp
    r1 = (1 if mode1==1 else 0)+zr/zs
    r2_target = ng/n*iss/r1
    zp2 = int(results['z_p2']['value'])
    assert zp2 in [c[1] for c in choose_planet(30,r2_target,mode2)], ('stage2 selection',zp2,choose_planet(30,r2_target,mode2))
    zr2 = 30+2*zp2
    r2 = (1 if mode2==1 else 0)+zr2/30
    sign1,sign2 = (1 if mode1==1 else -1),(1 if mode2==1 else -1)
    n1,n2 = n*r1*sign1,n*r1*sign1*r2*sign2
    t1,t2 = torque*e1/(r1*sign1),torque*e1*e2/(r1*sign1*r2*sign2)
    power = torque*2*math.pi*n/60
    for name,value in [('z_p',zp),('z_r',zr),('z_p2',zp2),('z_r2',zr2),('R_1',r1),('R_2',r2),('R_2_Ziel',r2_target),('i_1',1/r1),('i_2',1/r2),('i_gesamt',iss/r1/r2),('Drehsinn_1',sign1),('Drehsinn_2',sign2),('Drehzahlabweichung',abs(n2/iss)/ng-1),('Montagerest_1',(-zs)%3),('Montagerest_2',0)]: expected(name,value)
    for name,value in [('n_1',n1/60),('n_2',n2/60),('n_Generator',n2/iss/60)]: expected(name,value,'frequency')
    for name,value in [('P_Eingang',power),('P_1',power*e1),('P_2',power*e1*e2),('P_Generator',power*e1*e2*ess)]: expected(name,value,'power')
    for name,value in [('T_1',t1),('T_2',t2),('T_Generator',t2*ess*iss),('T_Sonne_Auslegung',torque/r1)]: expected(name,value,'torque')
    # SI beam/shaft calculations; no Calcpad unit conversion or evaluator reused.
    mmin = (2*1.25*torque/r1/(3*zs*width_factor*.3*160e6))**(1/3)
    m2min = (2*1.25*abs(t1/r2)/(3*30*width_factor*.3*160e6))**(1/3)
    mauto = next(mm/1000 for mm in PREFERRED if mm/1000 >= mmin-1e-14)
    m2auto = next(mm/1000 for mm in PREFERRED if mm/1000 >= m2min-1e-14)
    m = case.get('module',mauto*1000)/1000
    b,a = width_factor*m,m*(zs+zp)/2
    ft = (torque/r1)/(3*(m*zs/2))
    force = 2*1.25*ft
    span = b+2*.2*m+2*m
    db = (8*force*span/(math.pi*200e6))**(1/3)
    dd = (force*span**3/(48*210e9*(math.pi/64)*.00004))**.25
    dn = (force*span**2/(16*210e9*(math.pi/64)*.005))**.25
    pin = math.ceil(max(db,dd,dn)/2*1000-1e-12)/1000
    bore_s = math.ceil((16*(torque/r1)/(math.pi*60e6))**(1/3)/2*1000-1e-12)/1000
    hub = math.ceil((16*torque/(math.pi*60e6))**(1/3)/2*1000-1e-12)/1000+m
    inertia = math.pi*(2*pin)**4/64
    delta = force*span**3/(48*210e9*inertia)
    slope = force*span**2/(16*210e9*inertia)
    sigma_pin = (force*span/4)*(pin)/inertia
    sigma_f = 1.25*ft/(b*m*.3)
    wall = m*(zp/2-1.25)-(pin+m)
    lengths = dict(modul_min=mmin,modul2_min=m2min,modul_auto=mauto,modul2_auto=m2auto,modul=m,modul2=m2auto,b=b,a=a,d_s=m*zs,d_p=m*zp,d_r=m*zr,Spiel_axial=.2*m,Platte=2*m,Bolzenspanne=span,d_Bolzen_Biegung=db,d_Bolzen_Durchbiegung=dd,d_Bolzen_Neigung=dn,pin_r=pin,bore_p=pin+m,Wand_Planet=wall,bore_s=bore_s,hub_r=hub,ring_outer=m*(zr/2+6),Zahnflankenspiel=.02*m,Nachbarabstand=math.sqrt(3)*a-m*(zp+2),delta_Bolzen=delta)
    for name,value in lengths.items(): expected(name,value,'length')
    expected('I_Bolzen',inertia,'inertia')
    for name,value in [('F_t',ft),('F_Bolzen',force)]: expected(name,value,'force')
    for name,value in [('sigma_F',sigma_f),('sigma_Bolzen',sigma_pin)]: expected(name,value,'stress')
    for name,value in [('theta_Bolzen',slope),('S_Zahn',160e6/sigma_f),('S_Bolzen',200e6/sigma_pin),('S_Durchbiegung',.00004/delta),('S_Neigung',.005/slope),('Wandreserve',wall/m),('Montageindex',(zs+zr)/3),('n_eingang_rpm',n),('T_eingang_Nm',torque)]: expected(name,value)
    errors=[]
    for name,(value,dimension) in wanted.items():
        actual=results.get(name)
        if actual is None:
            errors.append(f'{name}: missing');continue
        scale,dim=SCALES.get(actual['unit'],(None,None))
        if dim != dimension:
            errors.append(f'{name}: wrong/unhandled unit {actual["unit"]!r} ({dimension})');continue
        converted=actual['value']*scale
        if not math.isclose(converted,value,rel_tol=2e-10,abs_tol=max(abs(value)*2e-12,1e-15)):
            errors.append(f'{name}: {converted} != {value} SI')
    # Independent physical balance: signed torque * angular speed = output power.
    for torque_name,speed_name,power_name in [('T_1','n_1','P_1'),('T_2','n_2','P_2'),('T_Generator','n_Generator','P_Generator')]:
        read=lambda key: results[key]['value']*SCALES[results[key]['unit']][0]
        assert math.isclose(read(torque_name)*2*math.pi*read(speed_name),read(power_name),rel_tol=1e-10)
    omitted=set(results)-set(wanted)-{'e','pi','\u03c0'}
    assert not omitted, ('unchecked results',omitted)
    return errors, len(wanted)


def cases():
    for v,l in itertools.product(range(1,5),repeat=2):
        yield dict(name=f'W{v}-A{l}-default',variant=v,layout=l)
        for key,values in [('torque',[100,1000,8000]),('ratio',[3.2,3.8,4.6]),('sun',[31,32,36])]:
            for value in values: yield dict(name=f'W{v}-A{l}-{key}{value}',variant=v,layout=l,**{key:value})
    for v in range(1,5):
        for key,value in [('width',10),('width',20),('module',30),('rpm',DATA[v][1]*1.1),('alpha',25)]:
            yield dict(name=f'W{v}-A{v}-{key}{value}',variant=v,layout=v,**{key:value})


def run(case):
    source=SOURCE.read_text(encoding='utf-8')
    for name,expr in [('Variante',case['variant']),('Aufbauvariante',case['layout'])]: source=edited(source,name,str(expr))
    for key,name,unit in [('torque','T_in','N*m'),('ratio','R_Ziel',''),('sun','z_s',''),('width','breitenfaktor',''),('module','modul','mm'),('rpm','n_in','/min'),('alpha','alpha_grad','')]:
        if key in case: source=edited(source,name,str(case[key])+unit)
    started=time.monotonic()
    p=subprocess.run([str(ROOT/'output/calcpad-local/CalcpadLocal.exe')],input=json.dumps({'source':source},ensure_ascii=False).encode('utf-8'),capture_output=True,timeout=15)
    data=json.loads(p.stdout.decode('utf-8-sig'))
    try:
        assert data.get('ok'),data.get('error')
        errors,count=reference(case,data['results'])
    except Exception as error:
        errors,count=[str(error)],0
    return dict(**case,passed=not errors,errors=errors,checks=count,elapsed_seconds=time.monotonic()-started,results=data.get('results',{}))


def main():
    OUT.mkdir(parents=True,exist_ok=True)
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        results=list(pool.map(run,cases()))
    report=dict(source_sha256=hashlib.sha256(SOURCE.read_bytes()).hexdigest(),cases=len(results),passed=sum(r['passed'] for r in results),scalar_checks=sum(r['checks'] for r in results),results=results,limitations='Arithmetic and preliminary models only; not certification, bearing selection or full assignment completion.')
    (OUT/'calculation-audit.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(json.dumps({k:v for k,v in report.items() if k!='results'}),flush=True)
    for case in results:
        if not case['passed']: print(json.dumps({k:v for k,v in case.items() if k!='results'}),flush=True)
    raise SystemExit(0 if report['passed']==len(results) else 1)


if __name__=='__main__': main()
