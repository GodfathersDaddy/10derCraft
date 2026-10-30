"""Private desktop instance for focused UI acceptance checks; never closes user apps."""
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import time

ROOT=Path(__file__).resolve().parents[2]

class Desktop:
    def __init__(self, folder, port=7894):
        self.out=Path(os.environ.get('SOLIDCRAFT_TEST_OUTPUT_ROOT',str(ROOT/'output')))/folder
        self.out.mkdir(parents=True,exist_ok=True)
        self.port=port
        self.token=secrets.token_hex(32)
    def __enter__(self):
        info=subprocess.STARTUPINFO()
        info.dwFlags|=subprocess.STARTF_USESHOWWINDOW
        info.wShowWindow=0
        self.log=(self.out/'desktop.log').open('w')
        self.process=subprocess.Popen([os.environ.get('SOLIDCRAFT_TEST_EXE',str(ROOT/'target/native-ui/ci/solidcraft.exe')),'--control',str(self.port)],startupinfo=info,
            env=dict(os.environ,SOLIDCRAFT_CONTROL_TOKEN=self.token,SOLIDCRAFT_CONTROL_DIR=str(self.out/'control'),SOLIDCRAFT_RECOVERY_DIR=str(self.out/'recovery'/self.token[:12]),SOLIDCRAFT_CRASH_DIR=str(self.out/'crash'/self.token[:12])),
            stdout=subprocess.DEVNULL,stderr=self.log)
        for _ in range(160):
            try:
                self.call('ui.inspect');return self
            except (OSError,RuntimeError):time.sleep(.25)
        self.process.terminate();self.log.close()
        raise RuntimeError('Desktop did not start')
    def call(self, method, **params):
        with socket.create_connection(('127.0.0.1',self.port),timeout=300) as sock:
            wire=sock.makefile('rwb')
            wire.write((json.dumps(dict(id=1,method=method,params=params,token=self.token))+'\n').encode())
            wire.flush();reply=json.loads(wire.readline())
            # A server timeout is still an executing command. Keep this connection
            # and collect its eventual reply; never resend a mutation.
            deadline=time.monotonic()+900
            while not reply.get('ok') and reply.get('running'):
                if time.monotonic()>deadline:
                    raise RuntimeError('Own desktop command still running after 15 minutes: '+method)
                time.sleep(1)
                wire.write((json.dumps(dict(id=2,method='control.pending',params={},token=self.token))+'\n').encode())
                wire.flush();pending=json.loads(wire.readline())
                if not pending.get('ok'):raise RuntimeError(pending)
                state=pending.get('result',{})
                if state.get('pending') is None:
                    raise RuntimeError('The control connection lost its pending reply: '+method)
                if state.get('done'):
                    reply=state['reply']
            if not reply.get('ok'):raise RuntimeError(reply)
            return reply.get('result')
    def cmd(self, command, **params):return self.call('engine.execute',command=command,params=params)
    def click(self, name):
        h=next(h for h in self.call('ui.handles',prefix=name) if h['name']==name)
        self.call('ui.click',x=h['x'],y=h['y']);time.sleep(.2)
    def text(self, handle, value):
        self.click(handle);self.call('ui.key',key='A',cmd=True);self.call('ui.text',text=value)
    def screenshot(self, name):self.call('ui.screenshot',path=str(self.out/name))
    def settled(self):
        for _ in range(120):
            state=self.call('ui.inspect')
            if state['dialog'] is None:return
            time.sleep(.15)
        raise RuntimeError(('Dialog did not finish',state))
    def __exit__(self, kind, value, traceback):
        try:
            try:
                if kind:self.screenshot('failure.png')
            except (OSError,RuntimeError):pass
            finally:
                self.call('app.quit',force=True)
                self.process.wait(timeout=20)
        finally:self.log.close()
