import asyncio, base64, hashlib, json, os, pathlib, shutil, signal, tempfile, time, subprocess

REPO = pathlib.Path('/Users/alancurrie/Projects/herdr-web-master')
HERDR = pathlib.Path('/tmp/herdr-pane-sprite/target/release/herdr')
QUEST = REPO / 'target/release/questmancer'
OUT = pathlib.Path('/tmp/questmancer-pane-header-acceptance')

async def case(graphics=True, crash=True):
    root = pathlib.Path(tempfile.mkdtemp(prefix='qhdr-', dir='/tmp'))
    sock, proxy_path = root/'api.sock', root/'proxy.sock'
    receipt = {'case':str(root), 'graphics':graphics, 'native_visual':'blocked: Computer Use denies Ghostty',
               'herdr_sha256':hashlib.sha256(HERDR.read_bytes()).hexdigest(),
               'questmancer_sha256':hashlib.sha256(QUEST.read_bytes()).hexdigest(), 'checks':[]}
    records, proxy_tasks, owned_panes, owned_pids = [], set(), [], set()
    server, proxy, linked, workspace = None, None, False, None
    report_seq = 0
    def save(): (root/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    def record(obj):
        records.append(obj)
        with (root/'headers.jsonl').open('a') as f: f.write(json.dumps(obj)+'\n')
    async def request(method, params=None, allow_error=False):
        reader, writer = await asyncio.open_unix_connection(str(sock))
        writer.write((json.dumps({'id':f'probe-{time.time_ns()}', 'method':method, 'params':params or {}})+'\n').encode())
        await writer.drain()
        reply = json.loads(await asyncio.wait_for(reader.readline(),5))
        writer.close(); await writer.wait_closed()
        if 'error' in reply and not allow_error: raise AssertionError((method,reply))
        return reply if allow_error else reply['result']
    async def forward(reader, writer):
        while data := await reader.read(65536):
            writer.write(data); await writer.drain()
    async def handle_proxy(reader, writer):
        task=asyncio.current_task(); proxy_tasks.add(task)
        remote_writer=None
        try:
            line=await reader.readline()
            if not line: return
            req=json.loads(line)
            remote_reader, remote_writer=await asyncio.open_unix_connection(str(sock))
            remote_writer.write(line); await remote_writer.drain()
            reply_line=await remote_reader.readline()
            if req.get('method','').startswith('pane.viewport_header.') or req.get('method') == 'session.snapshot':
                record({'at':time.time(), 'request':req, 'response':json.loads(reply_line)})
            writer.write(reply_line); await writer.drain()
            a=asyncio.create_task(forward(reader,remote_writer)); b=asyncio.create_task(forward(remote_reader,writer))
            done,pending=await asyncio.wait([a,b],return_when=asyncio.FIRST_COMPLETED)
            for item in pending: item.cancel()
            await asyncio.gather(a,b,return_exceptions=True)
        except (ConnectionError, asyncio.CancelledError): pass
        finally:
            if remote_writer: remote_writer.close()
            writer.close(); proxy_tasks.discard(task)
    def syncs():
        return [r for r in records if r['request']['method']=='pane.viewport_header.sync' and r['response'].get('result',{}).get('type')=='ok']
    async def wait_for(predicate, label, seconds=10):
        deadline=time.monotonic()+seconds
        while time.monotonic()<deadline:
            result=predicate()
            if result: return result
            if server and server.returncode is not None: raise AssertionError(('server exited',server.returncode))
            await asyncio.sleep(.05)
        raise AssertionError(('timeout',label,records[-2:]))
    async def wait_header(after, predicate, label):
        return await wait_for(lambda: next((r for r in syncs()[after:] if predicate(r['request']['params']['headers'])),None),label,15)
    async def report(pane,state,session='header-a'):
        nonlocal report_seq
        report_seq+=1
        await request('pane.report_agent_session', {'pane_id':pane,'source':'herdr:pi','agent':'pi','seq':report_seq,'agent_session_id':session,'session_start_source':'new'})
        report_seq+=1
        return await request('pane.report_agent',{'pane_id':pane,'source':'herdr:pi','agent':'pi',
                            'state':state,'seq':report_seq,'agent_session_id':session})
    async def open_plugin():
        result=await request('plugin.pane.open',{'plugin_id':'opsydyn.questmancer','entrypoint':'guild-hall',
                             'workspace_id':workspace,'focus':False})
        pane=result['plugin_pane']['pane']['pane_id']; owned_panes.append(pane)
        await wait_for(lambda:(root/'quest.pid').exists(),'publisher pid')
        pid=int((root/'quest.pid').read_text()); owned_pids.add(pid)
        return pane,pid
    save()
    try:
        fixture=root/'plugin'; (fixture/'herdr').mkdir(parents=True); (fixture/'target/release').mkdir(parents=True)
        shutil.copy(REPO/'herdr-plugin.toml',fixture/'herdr-plugin.toml')
        for name in ['control.sh','install.sh']: shutil.copy(REPO/'herdr'/name,fixture/'herdr'/name)
        (fixture/'target/release/questmancer').symlink_to(QUEST)
        (fixture/'herdr/run.sh').write_text(f'''#!/bin/sh
printf '%s\\n' "$$" > '{root}/quest.pid'
export HERDR_SOCKET_PATH='{proxy_path}'
exec '{REPO}/herdr/run.sh' "$@"
''')
        (fixture/'herdr/run.sh').chmod(0o755)
        (root/'pi.c').write_text('#include <unistd.h>\nint main(void) { sleep(300); return 0; }\n')
        subprocess.run(['clang', str(root/'pi.c'), '-o', str(root/'pi')], check=True)
        plain=root/'plain.sh'
        plain.write_text(f'''#!/bin/sh
printf '%s\\n' "$$" > '{root}/dummy.pid'
printf '%s\\n' 'QHDR_OWNED_TERMINAL_MARKER'
exec '{root}/pi' 300
'''); plain.chmod(0o755)
        config=root/'config.toml'
        config.write_text(f'''[terminal]
default_shell = "{plain}"
shell_mode = "non_login"
kitty_graphics = {str(graphics).lower()}
[ui]
pane_viewport_header = true
[update]
version_check = false
manifest_check = false
[session]
resume_agents_on_restore = false
''')
        env={k:v for k,v in os.environ.items() if not k.startswith('HERDR_')}
        env.update({'HERDR_SOCKET_PATH':str(sock),'HERDR_CONFIG_PATH':str(config),
                    'XDG_CONFIG_HOME':str(root/'cfg'),'XDG_STATE_HOME':str(root/'state'),
                    'TERM':'xterm-256color','COLORTERM':'truecolor'})
        log=(root/'server.log').open('wb')
        server=await asyncio.create_subprocess_exec(str(HERDR),'server',cwd=root,env=env,stdout=log,stderr=log)
        receipt['server_pid']=server.pid
        await wait_for(sock.exists,'server socket')
        proxy=await asyncio.start_unix_server(handle_proxy,path=str(proxy_path),limit=1<<20)
        await request('pane.viewport_header.info')
        created=await request('workspace.create',{'cwd':str(root),'label':'Owned header repair','focus':True})
        workspace=created['workspace']['workspace_id']
        snap=(await request('session.snapshot'))['snapshot']; receipt['initial_snapshot']=snap
        dummy=next(p['pane_id'] for p in snap['panes'] if p['workspace_id']==workspace)
        owned_panes.append(dummy)
        await wait_for(lambda:(root/'dummy.pid').exists(),'dummy process')
        owned_pids.add(int((root/'dummy.pid').read_text()))
        await asyncio.sleep(1)
        receipt['initial_report'] = await report(dummy,'working')
        receipt['after_report_snapshot'] = (await request('session.snapshot'))['snapshot']
        save()
        await request('plugin.link',{'path':str(fixture)}); linked=True
        managed,pid=await open_plugin()
        initial=await wait_header(0,lambda hs:len(hs)==1 and hs[0]['pane_id']==dummy and hs[0]['label'].endswith('working'),'initial header')
        assert initial['request']['params']['headers'][0]['pane_id']!=managed
        pixels=base64.b64decode(initial['request']['params']['headers'][0]['rgba_base64'])
        assert len(pixels)==384 and any(pixels[3::4])
        receipt['checks'].append('managed exclusion and authored 8x12 RGBA accepted by actual Herdr')
        for state,condition in [('idle','resting'),('blocked','seeks counsel'),('unknown','whereabouts unknown'),('working','working')]:
            start=len(syncs()); await report(dummy,state)
            await wait_header(start,lambda hs:len(hs)==1 and hs[0]['label'].endswith(condition),state)
        receipt['checks'].append('working idle blocked unknown live reports update the line')
        start=len(syncs()); stale=json.loads(json.dumps(syncs()[-1]['request']['params']))
        await report(dummy,'working','header-b')
        fresh=await wait_header(start,lambda hs:len(hs)==1 and hs[0]['agent_session']['value']=='header-b','session replacement')
        stale['sequence']=fresh['request']['params']['sequence']+1
        rejected=await request('pane.viewport_header.sync',stale,True)
        assert rejected['error']['code']=='stale_identity',rejected
        receipt['checks'].append('session replacement accepted; old session rejected')
        before=await request('pane.read',{'pane_id':dummy,'source':'recent_unwrapped','format':'text','lines':20})
        text=before['read']['text']; assert 'QHDR_OWNED_TERMINAL_MARKER' in text,text
        assert 'printf' not in text and 'seeks counsel' not in text and 'whereabouts' not in text,text
        receipt['checks'].append('dummy terminal output retains its marker and contains no injected header or printf command')
        start=len(syncs()); report_seq+=1
        await request('pane.release_agent',{'pane_id':dummy,'source':'herdr:pi','agent':'pi','seq':report_seq})
        await request('pane.close', {'pane_id':dummy}); owned_panes.remove(dummy)
        await wait_header(start,lambda hs:not hs,'closed pane clear')
        receipt['checks'].append('release followed by owned pane close clears the producer set')
        created=await request('workspace.create',{'cwd':str(root),'label':'Owned replacement','focus':False})
        replacement_workspace=created['workspace']['workspace_id']
        snap=(await request('session.snapshot'))['snapshot']
        dummy=next(p['pane_id'] for p in snap['panes'] if p['workspace_id']==replacement_workspace)
        owned_panes.append(dummy)
        await asyncio.sleep(1)
        owned_pids.add(int((root/'dummy.pid').read_text()))
        await report(dummy,'working','header-c'); await wait_header(start,lambda hs:len(hs)==1 and hs[0]['agent_session']['value']=='header-c','rejoin')
        start=len(syncs()); os.kill(pid,signal.SIGTERM)
        await wait_header(start,lambda hs:not hs,'graceful shutdown clear')
        snap=(await request('session.snapshot'))['snapshot']
        if any(p['pane_id']==managed for p in snap['panes']):
            await request('plugin.pane.close',{'pane_id':managed})
        owned_panes.remove(managed)
        workspace=replacement_workspace
        receipt['checks'].append('actual publisher SIGTERM sends acknowledged empty sync')
        if crash:
            (root/'quest.pid').unlink(missing_ok=True)
            start=len(syncs()); managed,pid=await open_plugin()
            active=await wait_header(start,lambda hs:len(hs)==1 and hs[0]['agent_session']['value']=='header-c','restart')
            receipt['checks'].append('publisher restart negotiates and republishes')
            # Wait for a renewal to prove the ten-second timer is live, not an animation-frame path.
            renewed=await wait_header(len(syncs()),lambda hs:len(hs)==1,'lease renewal',)
            receipt['checks'].append('active lease renewed on the actual socket')
            os.kill(pid,signal.SIGKILL)
            probe={'source':'test:expiry-probe','sequence':1,'headers':json.loads(json.dumps(active['request']['params']['headers']))}
            probe['headers'][0]['rgba_base64']=None
            conflict=await request('pane.viewport_header.sync',probe,True)
            assert conflict['error']['code']=='source_conflict',conflict
            await asyncio.sleep(31)
            await request('pane.viewport_header.sync',probe)
            await request('pane.viewport_header.sync',{'source':'test:expiry-probe','sequence':2,'headers':[]})
            receipt['checks'].append('crashed publisher lease expires on the actual server within 31 seconds')
        receipt['passed']=True
    except BaseException as exc:
        receipt['error']=repr(exc); receipt['passed']=False
        raise
    finally:
        errors=[]
        if server and server.returncode is None and sock.exists():
            for pane in list(owned_panes):
                try:
                    report_seq+=1
                    await request('pane.release_agent',{'pane_id':pane,'source':'herdr:pi','agent':'pi','seq':report_seq},True)
                    await request('pane.close',{'pane_id':pane},True)
                except Exception as exc: errors.append(repr(exc))
            if linked:
                try: await request('plugin.unlink',{'plugin_id':'opsydyn.questmancer'})
                except Exception as exc: errors.append(repr(exc))
            try:
                receipt['final_snapshot']=(await request('session.snapshot'))['snapshot']
                receipt['final_plugins']=await request('plugin.list')
                await request('server.stop')
            except Exception as exc: errors.append(repr(exc))
            try: await asyncio.wait_for(server.wait(),10)
            except asyncio.TimeoutError:
                server.terminate(); await server.wait(); errors.append('owned server required terminate')
        if proxy:
            proxy.close(); await proxy.wait_closed()
        for task in list(proxy_tasks): task.cancel()
        await asyncio.gather(*list(proxy_tasks),return_exceptions=True)
        await asyncio.sleep(.2)
        alive=[]
        for pid in owned_pids:
            try: os.kill(pid,0); alive.append(pid)
            except ProcessLookupError: pass
        receipt['cleanup']={'server_exit':server.returncode if server else None,'socket_removed':not sock.exists(),
                            'owned_pids_still_alive':alive,'errors':errors}
        receipt['owned_pids']=sorted(owned_pids); save()
        print(json.dumps({'case':str(root),'passed':receipt.get('passed'),'checks':receipt['checks'],'cleanup':receipt['cleanup'],'error':receipt.get('error')}),flush=True)
        (OUT/('latest-graphics.json' if graphics else 'latest-text.json')).write_text(json.dumps(receipt,indent=2)+'\n')

async def main():
    await case(True,True)
    await case(False,False)
asyncio.run(main())
