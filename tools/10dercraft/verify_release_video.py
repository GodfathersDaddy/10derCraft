"""Exercise actual native projects and decode the new video independently."""
from pathlib import Path
import os,sys,time,json,struct,hashlib,shutil
sys.path.insert(0,str(Path(__file__).resolve().parent))
from cad_test_client import Desktop,ROOT
from ui_helpers import handle,wait_handle
sys.path.insert(0,str(ROOT/'output/research-plot-packages'))
from PIL import Image,ImageChops
from io import BytesIO
os.environ.setdefault('SOLIDCRAFT_TEST_EXE',str(ROOT/'target/native-ui/ci/solidcraft.exe'))
os.environ['SOLIDCRAFT_SIMULATION_PYTHON']=str(ROOT/'simulation/.venv/Scripts/python.exe')
def decode(path,fps,dimensions):
    b=path.read_bytes();assert b[:4]==b'RIFF' and b[8:12]==b'AVI '
    assert struct.unpack_from('<I',b,4)[0]==len(b)-8
    def chunks(start,end):
        while start<end:
            tag=b[start:start+4];size=struct.unpack_from('<I',b,start+4)[0]
            assert start+8+size<=end
            yield tag,start+8,size
            start+=8+size+(size%2)
        assert start==end
    top=list(chunks(12,len(b)));hdrl=next((p,n) for tag,p,n in top if tag==b'LIST' and b[p:p+4]==b'hdrl')
    headers=list(chunks(hdrl[0]+4,hdrl[0]+hdrl[1]));avih=next(p for tag,p,n in headers if tag==b'avih')
    values=struct.unpack_from('<14I',b,avih);assert values[0]==1000000//fps and tuple(values[8:10])==dimensions
    movi,n=next((p,n) for tag,p,n in top if tag==b'LIST' and b[p:p+4]==b'movi')
    frames=list(chunks(movi+4,movi+n));assert len(frames)==values[4]
    idx,sz=next((p,n) for tag,p,n in top if tag==b'idx1');assert sz==16*len(frames)
    images=[]
    for i,(tag,p,n) in enumerate(frames):
        assert tag==b'00dc'
        fourcc,flags,offset,size=struct.unpack_from('<4sIII',b,idx+i*16)
        assert fourcc==tag and flags==16 and movi+offset==p-8 and size==n
        image=Image.open(BytesIO(b[p:p+n]));image.load();assert image.size==dimensions
        images.append(image.convert('RGB'))
    assert any(ImageChops.difference(images[0],im).getbbox() for im in images[1:]),'video must contain actual movement'
    images[0].save(path.with_suffix('.first.png'));images[-1].save(path.with_suffix('.last.png'))
    return dict(bytes=len(b),frames=len(frames),fps=fps,decoded_frames=len(images))
def main():
    with Desktop('release-video',7984) as d:
        catalog=json.loads((ROOT/'10derCraft.examples.json').read_text(encoding='utf-8'))
        checks=[]
        for item in catalog['examples']:
            path=ROOT/item['path'];before=hashlib.sha256(path.read_bytes()).hexdigest()
            d.cmd('doc.open',path=str(path));d.settled()
            assert hashlib.sha256(path.read_bytes()).hexdigest()==before
            checks.append(dict(id=item['id'],opened=True))
            print('opened',item['id'],flush=True)
        video_checks=[]
        for id in ['weighted-pendulum','robot-drag-workflow']:
            item=next(v for v in catalog['examples'] if v['id']==id)
            d.cmd('doc.open',path=str(ROOT/item['path']));d.settled()
            state=d.cmd('mks.job.state')
            if not state['result_current']:
                settings=state['data']['job']['settings'];assert isinstance(settings,dict)
                d.cmd('mks.job.run',**settings)
            path=d.out/(id+'.avi')
            started=time.monotonic()
            result=d.cmd('mks.result.video',path=str(path),width=320,height=180,fps=15,overwrite=True)
            report=decode(path,15,(320,180));report.update(id=id,seconds=time.monotonic()-started)
            assert report['frames']==result['frames']
            digest=hashlib.sha256(path.read_bytes()).hexdigest()
            try:d.cmd('mks.result.video',path=str(path));raise AssertionError('existing video overwritten')
            except RuntimeError:pass
            assert hashlib.sha256(path.read_bytes()).hexdigest()==digest
            invalid=d.out/'invalid.avi'
            try:d.cmd('mks.result.video',path=str(invalid),fps=0);raise AssertionError('invalid FPS accepted')
            except RuntimeError:pass
            assert not invalid.exists()
            assert d.cmd('mks.job.state')['result_current']
            video_checks.append(report);print(report,flush=True)
        d.call('ui.window',action='restore');d.call('ui.resize',width=1800,height=1100);time.sleep(1)
        def click(name):
            h=handle(d,name);d.call('ui.move',x=h['x'],y=h['y']);time.sleep(.15);d.call('ui.click',x=h['x'],y=h['y']);time.sleep(.3)
        click('project:mks');wait_handle(d,'mks:library')
        ui_path=d.out/f'Robot-UI-Export-{time.time_ns()}.avi'
        assert not ui_path.exists()
        click('mks:video:open');wait_handle(d,'mks:video:path');d.text('mks:video:path',str(ui_path))
        d.screenshot('Videoexport-Dialog.png');click('mks:video:save')
        deadline=time.monotonic()+240
        while time.monotonic()<deadline:
            d.call('ui.inspect') # remains responsive while exporting
            if ui_path.exists():break
            time.sleep(.5)
        assert ui_path.is_file()
        ui_video=decode(ui_path,30,(640,360))
        shutil.copy2(ui_path,d.out/'Robot-UI-Export.avi')
        time.sleep(.5);d.screenshot('Videoexport-Gespeichert.png')
        report=dict(passed=True,projects=checks,videos=video_checks,ui_video=ui_video)
        (d.out/'verification.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report),flush=True)

if __name__=='__main__': main()
