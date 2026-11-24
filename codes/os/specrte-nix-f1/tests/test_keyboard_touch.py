"""Compare the shipped C scorer with the user's original JavaScript model.
Run: uv run --with quickjs python tests/test_keyboard_touch.py
"""
import ctypes as c
import json
import math
from pathlib import Path
import random
import subprocess
import tempfile
import unittest
import quickjs

ROOT=Path(__file__).resolve().parents[1]
class Key(c.Structure):
    _fields_=[('x',c.c_double),('y',c.c_double),('w',c.c_double),('h',c.c_double),('symbol',c.c_char_p),('command',c.c_int)]

class Scorer(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp=tempfile.TemporaryDirectory()
        source=Path(cls.tmp.name)/'test.c'
        source.write_text('#include "spectre-touch.h"\nint choose(const SpectreTouchKey*k,size_t n,double x,double y,const char*p,int a){return spectre_touch_choose(k,n,x,y,p,a);}\nvoid prefix(char*out,const char*t,size_t n){spectre_prefix_set(out,t,n);}\nsize_t erase_start(const unsigned int*p,size_t n){return spectre_erase_start(p,n);}\n')
        binary=Path(cls.tmp.name)/'test.so'
        subprocess.run(['cc','-std=c11','-Wall','-Wextra','-Werror','-shared','-fPIC',str(source),'-I',str(ROOT/'keyboard/typing'),'-lm','-o',str(binary)],check=True)
        cls.lib=c.CDLL(str(binary));cls.lib.choose.argtypes=[c.POINTER(Key),c.c_size_t,c.c_double,c.c_double,c.c_char_p,c.c_int]
        cls.lib.prefix.argtypes=[c.c_char_p,c.c_char_p,c.c_size_t]
        cls.lib.erase_start.argtypes=[c.POINTER(c.c_uint),c.c_size_t];cls.lib.erase_start.restype=c.c_size_t
        cls.js=quickjs.Context();cls.js.eval((ROOT/'keyboard/typing/TouchModel.js').read_text())
    @classmethod
    def tearDownClass(cls):cls.tmp.cleanup()
    def test_reference_parity(self):
        random.seed(845)
        for language in ('en','fa'):
            keys=json.loads(self.js.eval(f'JSON.stringify(layout(440,286,false,false,"{language}"))'))
            array=(Key*len(keys))(*[Key(k['x'],k['y'],k['w'],k['h'],k['label'].encode(),bool(k['action'])) for k in keys])
            self.js.eval('var testKeys='+json.dumps(keys,ensure_ascii=False))
            for _ in range(1500):
                x=random.uniform(-15,455);y=random.uniform(-15,301);text=random.choice(['','hel','th','ke','wor','xyz','سلام']);adaptive=random.choice([True,False])
                prefix=c.create_string_buffer(64);raw=text.encode();self.lib.prefix(prefix,raw,len(raw))
                chosen=self.lib.choose(array,len(keys),x,y,prefix.value,adaptive)
                expected=self.js.eval(f'(function(){{var r=choose(testKeys,{x},{y},{json.dumps(text)},{str(adaptive).lower()});return r?testKeys.indexOf(r.key):-1;}})()')
                self.assertEqual(chosen,expected,(language,x,y,text,adaptive))
    def test_action_keys_never_become_letters(self):
        keys=(Key*2)(Key(0,0,40,60,b'q',0),Key(40,0,40,60,b'KEY_BACKSPACE',1))
        for prefix in (b'hel',b'th',b'wor'):
            for x in (40,40.01,60,79.999):self.assertEqual(self.lib.choose(keys,2,x,20,prefix,1),1)
    def test_prefix_reset_and_cursor(self):
        out=c.create_string_buffer(64)
        for text,expected in [('Hello','hello'),('hello world','world'),('سلام',''),('ab1cd','cd'),('a'*100,'')]:
            raw=text.encode();self.lib.prefix(out,raw,len(raw));self.assertEqual(out.value.decode(),expected)
        self.lib.prefix(out,b'hello world',2);self.assertEqual(out.value,b'he')
        self.lib.prefix(out,b'',0);self.assertEqual(out.value,b'')
    def test_emoji_deletion_matches_reference(self):
        samples=['','hello','hello👩‍💻','👍🏽','🇮🇷','🇮🇷🇩🇪','🇮🇷🇩','é','بَ','👨‍👩‍👧‍👦','1️⃣','a👩‍🦼‍➡️']
        for text in samples:
            points=(c.c_uint*len(text))(*map(ord,text))
            actual=text[:self.lib.erase_start(points,len(text))]
            expected=self.js.eval('erase('+json.dumps(text)+')')
            self.assertEqual(actual,expected,text)
    def test_nonfinite_input_is_rejected(self):
        keys=(Key*1)(Key(0,0,40,60,b'q',0))
        for x in (math.nan,math.inf,-math.inf):self.assertEqual(self.lib.choose(keys,1,x,20,b'',1),-1)

if __name__=='__main__':unittest.main()
