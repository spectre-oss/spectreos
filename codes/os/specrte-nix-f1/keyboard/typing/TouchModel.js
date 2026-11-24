// Mathematical port of the non-correlated native scoring branch, not its trained parameters.
// APK VAs: distance 0x1a0a648; log score 0x1a0a6e4/0x1a0a77c.
function axisDistance(value, center, plateau, lower, upper) {
    if (!plateau) return value - center;
    if (value >= lower && value <= upper) return 0;
    return value < lower ? lower - value : value - upper;
}
function nativeDistance(x, y, p) {
    var dx = axisDistance(x,p.cx,p.plateauX,p.left,p.right);
    var dy = axisDistance(y,p.cy,p.plateauY,p.top,p.bottom);
    return dx*dx*p.wx + dy*dy*p.wy;
}
function nativeLogScore(x,y,p) {
    if (!(p.amplitude > 0 && p.wx > 0 && p.wy > 0)) throw Error('Invalid Gaussian parameters');
    return Math.log(p.amplitude) - 0.5*nativeDistance(x,y,p);
}
// Prototype calibration. These widths and priors are NOT recovered Gboard weights.
function parameters(key) {
    var cx=key.x+key.w/2, cy=key.y+key.h/2;
    return {cx:cx,cy:cy,plateauX:true,plateauY:true,
        left:cx-key.w*0.1,right:cx+key.w*0.1,top:cy-key.h*0.1,bottom:cy+key.h*0.1,
        wx:1/Math.pow(key.w*0.38,2),wy:1/Math.pow(key.h*0.38,2),amplitude:1};
}
var demoWords = 'the there their this that these those hello help keyboard key keys touch type typing text test world work word words with what when where which would you your yes have has how home good great google gboard number numbers new next not now one only on of open phone please probability project first from for fix and are all app as at after back backspace can change create click code could do done delete did each exact english enter example function get go has here input into is it its just keep layout letters like local make model more my no other out people press right same see select separate shift should show some space stable start than them then they time to two up use version very want was we were will with would'.split(' ');
function priors(text) {
    var result={}, alphabet='abcdefghijklmnopqrstuvwxyz', prefix=text.toLowerCase().split(/[^a-z]/).pop();
    for(var i=0;i<alphabet.length;i++) result[alphabet[i]]=1;
    if(prefix.length) for(i=0;i<demoWords.length;i++) {
        var word=demoWords[i];
        if(word.indexOf(prefix)===0 && word.length>prefix.length) result[word[prefix.length]]+=4;
    }
    return result;
}
function choose(keys,x,y,text,adaptive) {
    var prior=priors(text), scored=[];
    // Commands stay deterministic, so probability cannot turn delete into a letter.
    for(var i=0;i<keys.length;i++) {
        var k=keys[i];
        if(k.action && x>=k.x && x<k.x+k.w && y>=k.y && y<k.y+k.h)
            return {key:k,score:0,runner:null};
    }
    for(i=0;i<keys.length;i++) {
        k=keys[i]; if(k.action) continue;
        if(Math.abs(y-(k.y+k.h/2))>k.h || Math.abs(x-(k.x+k.w/2))>k.w*1.5)continue;
        var score=nativeLogScore(x,y,parameters(k));
        if(adaptive)score+=0.7*Math.log(prior[k.label.toLowerCase()]||1);
        scored.push({key:k,score:score});
    }
    scored.sort(function(a,b){return b.score-a.score;});
    if(!scored.length)return null;
    return {key:scored[0].key,score:scored[0].score,runner:scored.length>1?scored[1]:null};
}
function layout(width,height,symbols,shifted,language) {
    var rows=symbols?(shifted?['~`|•√π÷×¶∆','£¢€¥^°={}','\%©®™✓[]']:['1234567890','@#$%&-+()','*"\':;!?']):['qwertyuiop','asdfghjkl','zxcvbnm'];
    var fa=language==='fa';
    if(fa&&!symbols) rows=['ضصثقفغعهخحج','شسیبلاتنمکگ','ظطژزرذدپوچ'];
    if(fa&&symbols&&!shifted) rows=['۱۲۳۴۵۶۷۸۹۰','@#٪&ـ+()','*"،؛:؟!'];
    var w=width/10,h=height/4,keys=[];
    var persianLetters=fa&&!symbols;
    function add(label,x,y,kw,action){keys.push({label:label,x:x,y:y,w:kw,h:h,action:action||''});}
    for(var r=0;r<3;r++) {
        var row=rows[r],rw=persianLetters?width/11:Math.min(w,(r===2?width-3*w:width)/row.length),start=persianLetters?0:(width-row.length*rw)/2;
        for(var c=0;c<row.length;c++)add(!symbols&&shifted?row[c].toUpperCase():row[c],start+c*rw,r*h,rw);
    }
    if(!persianLetters) add(symbols?(shifted?'?123':'=\\<'):'⇧',0,2*h,1.5*w,'shift');add('⌫',persianLetters?width*10/11:8.5*w,2*h,persianLetters?width/11:1.5*w,'delete');
    add(symbols?(fa?'اب‌ج':'ABC'):'?123',0,3*h,1.5*w,'symbols');add(fa?'،':',',1.5*w,3*h,w);
    add('◎',2.5*w,3*h,w,'language');add(fa?'فارسی':'English',3.5*w,3*h,fa?3*w:4*w,'space');if(fa)add('‌',6.5*w,3*h,w,'zwnj');add('.',7.5*w,3*h,w);add('↵',8.5*w,3*h,1.5*w,'enter');
    return keys;
}
var alternatives={'ا':['ٱ','ء','آ','أ','إ'],'ه':['ه‍','هٔ','ة','ۀ'],'ی':['ئ','ي','ى'],'ت':['ة'],'ک':['ك'],'و':['ؤ'],'.':['؟','!','ً','ّ','ٌ','ٍ','ْ','ٔ','ٕ','َ','ـ','ُ','ِ','ٓ','ٰ','ٖ']};
function holdAction(key,language,symbols){
 if(key.action==='language'||key.label===','||key.label==='،')return 'emoji';
 if(key.action==='space')return 'languages';
 if(key.action==='shift'&&language!=='fa'&&!symbols)return 'caps';
 if(language==='fa'&&!symbols&&alternatives[key.label])return 'alternatives';
 return '';
}
// Delete a displayed emoji cluster, including joiners, modifiers and flag pairs.
function erase(text) {
    var points=[], offsets=[];
    for(var i=0;i<text.length;i++){offsets.push(i);var c=text.charCodeAt(i);if(c>=0xd800&&c<=0xdbff&&i+1<text.length){var low=text.charCodeAt(i+1);if(low>=0xdc00&&low<=0xdfff){c=0x10000+((c-0xd800)<<10)+low-0xdc00;i++;}}points.push(c);}
    function extension(c){return c===0xfe0f||c===0xfe0e||c===0x20e3||(c>=0x1f3fb&&c<=0x1f3ff)||(c>=0x300&&c<=0x36f)||(c>=0x64b&&c<=0x65f)||c===0x670;}
    var n=points.length-1;if(n<0)return '';
    if(points[n]>=0x1f1e6&&points[n]<=0x1f1ff){var count=0;for(var j=n;j>=0&&points[j]>=0x1f1e6&&points[j]<=0x1f1ff;j--)count++;if(count%2===0)n--;}
    else {while(n>0&&extension(points[n]))n--;while(n>1&&points[n-1]===0x200d){n-=2;while(n>0&&extension(points[n]))n--;}}
    return text.slice(0,offsets[n]);
}
if(typeof module!=='undefined')module.exports={axisDistance:axisDistance,nativeDistance:nativeDistance,nativeLogScore:nativeLogScore,parameters:parameters,priors:priors,choose:choose,layout:layout,erase:erase,alternatives:alternatives,holdAction:holdAction};
