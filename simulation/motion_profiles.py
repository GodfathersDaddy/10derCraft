"""Shared prescribed joint motion and its analytic time derivative."""
from hashlib import sha256
from pathlib import Path
import math

SOURCE_SHA256 = sha256(Path(__file__).read_bytes()).hexdigest()


def joint_motion(drive, kind, duration, frozen=None):
    amplitude = float(drive['amplitude_si'])
    ramp = float(drive['ramp_s'])
    frequency = float(drive.get('frequency_hz', 1.))
    profile = drive.get('profile', 'cosine')
    if profile == 'half_cosine_displacement_then_hold':
        profile = 'cosine'
    if (not math.isfinite(amplitude) or not math.isfinite(ramp) or not 0 < ramp <= duration
            or not math.isfinite(frequency) or not .01 <= frequency <= 20
            or profile not in ('cosine', 'quintic', 'sine') or kind not in ('revolute', 'slider')):
        raise ValueError('invalid prescribed joint motion')
    coordinate = 5 if kind == 'revolute' else 2

    def sample(t):
        if frozen is not None and frozen():
            return 0., 0.
        phase = min(max(t / ramp, 0.), 1.)
        phase_rate = 1. / ramp if 0 < t < ramp else 0.
        if profile == 'cosine':
            value = .5 * (1. - math.cos(math.pi * phase))
            rate = .5 * math.pi * math.sin(math.pi * phase) * phase_rate
        else:
            smooth = phase ** 3 * (10. + phase * (-15. + 6. * phase))
            smooth_rate = 30. * phase ** 2 * (1. - phase) ** 2 * phase_rate
            value, rate = smooth, smooth_rate
            if profile == 'sine':
                omega = 2. * math.pi * frequency
                value = smooth * math.sin(omega * t)
                rate = smooth_rate * math.sin(omega * t) + smooth * omega * math.cos(omega * t)
        return amplitude * value, amplitude * rate

    def offset(mbs, t, item_number, parameters):
        result = [0.] * 6
        result[coordinate] = sample(t)[0]
        return result

    def velocity(mbs, t, item_number, parameters):
        result = [0.] * 6
        result[coordinate] = sample(t)[1]
        return result

    return offset, velocity
