const {chromium}=require('/Users/alancurrie/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const fs=require('fs');
(async()=>{
 const browser=await chromium.launch({headless:true});
 const page=await browser.newPage({viewport:{width:800,height:1100},deviceScaleFactor:1});
 const errors=[];page.on('pageerror',e=>errors.push(String(e)));
 await page.goto('file:///tmp/quest-board-preview.html');
 const frame=page.frames().find(f=>f!==page.mainFrame());
 if(!frame)throw Error('Missing preview frame');
 const loc=id=>frame.locator('#qb-'+id);
 const check=async(id,expected)=>{const actual=await loc(id).textContent();if(!actual.includes(expected))throw Error(id+': '+actual);};
 await check('count','1 waiting');
 await loc('progress').click();await check('notice','decision remains open');
 await loc('scenario').selectOption('exited');await check('worker','session exited');if(!await loc('observe').isDisabled())throw Error('observe enabled after exit');await check('count','1 waiting');
 await loc('scenario').selectOption('live');await loc('seen').click();await check('local','seen');await check('count','1 waiting');
 await loc('defer').click();await check('count','1 set aside');
 await loc('scenario').selectOption('restart');await check('source','awaiting verification');await check('local','set aside');
 await loc('jump').click();await check('notice','No eligible summons');
 await loc('time').click();await check('count','1 waiting');
 await loc('scenario').selectOption('resolved');await check('count','0 waiting');await check('quest','active');await check('local','Resolved');
 await loc('scenario').selectOption('empty');await check('title','Campaign work');if(!await loc('inventory').isHidden())throw Error('empty inventory visible');
 const captures=[];
 for(const [name,width,state] of [['open',800,'live'],['worker-exited',800,'exited'],['restart',800,'restart'],['offline',800,'offline'],['resolved',800,'resolved'],['empty',800,'empty'],['narrow',360,'live'],['minimum',320,'exited']]){
  await page.setViewportSize({width,height:1300});await loc('scenario').selectOption(state);
  const overflow=await frame.evaluate(()=>document.documentElement.scrollWidth>window.innerWidth);
  if(overflow)throw Error('Horizontal overflow '+name);
  const p='/Users/alancurrie/Projects/herdr-web-master/docs/design/reviews/2026-09-30-quest-board/'+name+'.png';
  await page.screenshot({path:p,fullPage:true});captures.push({name,width,state});
 }
 if(errors.length)throw Error(errors.join('\n'));
 fs.writeFileSync('/Users/alancurrie/Projects/herdr-web-master/docs/design/reviews/2026-09-30-quest-board/verification.json',JSON.stringify({kind:'concept-browser-only',checks:['progress preserves decision','exit disables observe and preserves decision','seen retains eligibility','defer retained through concept restart','expired defer re-enters queue','source resolution removes obligation and preserves active quest','empty board available','no horizontal overflow at 320/360/800','no browser errors'],captures},null,2)+'\n');
 console.log('Concept interaction checks passed; eight viewport/state captures saved.');await browser.close();
})().catch(e=>{console.error(e);process.exitCode=1;});
