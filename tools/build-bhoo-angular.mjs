// Build the optional, genuine Angular Material reference without a CLI or vendored node_modules.
import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import {execFileSync} from 'node:child_process';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const source=path.join(root,'codes/apps/bhoo-web/angular-reference');
const cache=path.join(root,'context/build/bhoo-angular');
const out=path.join(root,'codes/apps/bhoo-web/site/angular');
await fs.mkdir(cache,{recursive:true});await fs.mkdir(out,{recursive:true});
for(const name of ['package.json','package-lock.json'])await fs.copyFile(path.join(source,name),path.join(cache,name));
const manifest=await fs.readFile(path.join(source,'package-lock.json'),'utf8');
let installed='';try{installed=await fs.readFile(path.join(cache,'.installed-lock'),'utf8');}catch{}
if(installed!==manifest){
  execFileSync('npm',['ci','--ignore-scripts','--no-audit','--no-fund','--cache',path.join(cache,'npm-cache')],{cwd:cache,stdio:'inherit'});
  await fs.writeFile(path.join(cache,'.installed-lock'),manifest);
}
const require=createRequire(path.join(cache,'package.json'));
const esbuild=require('esbuild');const sass=require('sass');
await fs.copyFile(path.join(source,'reference.ts'),path.join(cache,'reference.ts'));
await esbuild.build({entryPoints:[path.join(cache,'reference.ts')],bundle:true,format:'esm',platform:'browser',target:'es2022',minify:true,legalComments:'external',outfile:path.join(out,'reference.js'),define:{ngDevMode:'false',ngJitMode:'true'}});
const css=sass.compile(path.join(source,'theme.scss'),{loadPaths:[path.join(cache,'node_modules')],style:'compressed'});
await fs.writeFile(path.join(out,'theme.css'),css.css);
// Ship the license of every package actually retained by esbuild, including transitive dependencies.
const meta=await esbuild.build({entryPoints:[path.join(cache,'reference.ts')],bundle:true,write:false,metafile:true,format:'esm',minify:true,define:{ngDevMode:'false',ngJitMode:'true'},logLevel:'silent'});
const packages=new Map();
for(const input of Object.keys(meta.metafile.inputs)){
  let dir=path.dirname(path.resolve(input));
  while(dir.startsWith(path.join(cache,'node_modules'))){
    try{const pkg=JSON.parse(await fs.readFile(path.join(dir,'package.json'),'utf8'));if(pkg.name&&pkg.version){packages.set(pkg.name,{dir,pkg});break;}}catch{}
    dir=path.dirname(dir);
  }
}
let notices='# Bundled Angular reference dependencies\n\nThese packages are used only by the comparison reference, not by libbhoo.\n\n';
const inventory=[];
for(const {dir,pkg} of [...packages.values()].sort((a,b)=>a.pkg.name.localeCompare(b.pkg.name))){
  const files=(await fs.readdir(dir)).filter(name=>/^(license|licence|notice|copyright)/i.test(name));
  if(!files.length)throw Error('Missing license for '+pkg.name);
  inventory.push({name:pkg.name,version:pkg.version,license:pkg.license,repository:pkg.repository});
  notices+=`## ${pkg.name} ${pkg.version} (${pkg.license})\n\n`;
  for(const file of files)notices+=await fs.readFile(path.join(dir,file),'utf8')+'\n\n';
}
await fs.writeFile(path.join(out,'THIRD_PARTY_LICENSES.txt'),notices);
await fs.writeFile(path.join(out,'versions.json'),JSON.stringify(inventory,null,2)+'\n');
console.log(`Angular Material reference: ${inventory.length} licensed packages; ${await fs.stat(path.join(out,'reference.js')).then(s=>s.size.toLocaleString())} byte JS bundle.`);
