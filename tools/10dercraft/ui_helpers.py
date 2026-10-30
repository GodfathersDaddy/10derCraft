"""Focused acceptance and real before/after screenshots of the engineering workspaces."""
import json, os, sys, time
from cad_test_client import Desktop, ROOT
def wait_handle(d, prefix, timeout=45):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        found=d.call('ui.handles',prefix=prefix)
        if found:return found
        time.sleep(.2)
    raise RuntimeError('Missing handle: '+prefix)

def handle(d,name):
    return next(h for h in wait_handle(d,name) if h['name']==name)

def double_click(d,name):
    h=handle(d,name)
    d.call('ui.click',x=h['x'],y=h['y'])
    d.call('ui.click',x=h['x'],y=h['y'])
    time.sleep(.5)

