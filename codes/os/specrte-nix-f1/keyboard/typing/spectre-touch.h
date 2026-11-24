/* Port of the user-supplied TouchModel.js prototype, not trained Gboard data. */
#pragma once
#include <math.h>
#include <stddef.h>
#include <string.h>
#include <ctype.h>

typedef struct {
  double x, y, w, h;
  const char *symbol;
  int command;
} SpectreTouchKey;

static const char spectre_demo_words[] =
"the there their this that these those hello help keyboard key keys touch type typing text test world work word words with what when where which would you your yes have has how home good great google gboard number numbers new next not now one only on of open phone please probability project first from for fix and are all app as at after back backspace can change create click code could do done delete did each exact english enter example function get go has here input into is it its just keep layout letters like local make model more my no other out people press right same see select separate shift should show some space stable start than them then they time to two up use version very want was we were will with would";

/* Only retain the current ASCII word, with a fixed bound and no disk/log I/O. */
static inline void spectre_prefix_set(char prefix[64], const char *text, size_t len) {
  size_t start = len;
  while (start && ((text[start-1]>='a'&&text[start-1]<='z') ||
                   (text[start-1]>='A'&&text[start-1]<='Z'))) start--;
  size_t n = len-start;
  prefix[0] = 0;
  if (n >= 64) return;
  for (size_t i=0;i<n;i++) prefix[i]=(char)tolower((unsigned char)text[start+i]);
  prefix[n]=0;
}
static inline double spectre_next_prior(const char *prefix, const char *symbol) {
  double prior=1;
  if (!prefix || !*prefix || !symbol || !*symbol || symbol[1]) return prior;
  size_t n=strlen(prefix);
  unsigned char next=(unsigned char)tolower((unsigned char)*symbol);
  const char *word=spectre_demo_words;
  while (*word) {
    const char *end=strchr(word,' ');
    size_t len=end?(size_t)(end-word):strlen(word);
    if (len>n && !memcmp(word,prefix,n) && (unsigned char)word[n]==next) prior+=4;
    if (!end) break;
    word=end+1;
  }
  return prior;
}
static inline double spectre_axis_distance(double value,double lo,double hi) {
  return value<lo?lo-value:value>hi?value-hi:0;
}
static inline double spectre_touch_score(const SpectreTouchKey *k,double x,double y,
                                          const char *prefix,int adaptive) {
  if (!(k->w>0 && k->h>0)) return -INFINITY;
  double cx=k->x+k->w/2,cy=k->y+k->h/2;
  double dx=spectre_axis_distance(x,cx-k->w*.1,cx+k->w*.1);
  double dy=spectre_axis_distance(y,cy-k->h*.1,cy+k->h*.1);
  double score=-.5*(dx*dx/pow(k->w*.38,2)+dy*dy/pow(k->h*.38,2));
  if (adaptive) score+=.7*log(spectre_next_prior(prefix,k->symbol));
  return score;
}
static inline int spectre_touch_choose(const SpectreTouchKey *keys,size_t count,
                                       double x,double y,const char *prefix,int adaptive) {
  if (!isfinite(x)||!isfinite(y)) return -1;
  for (size_t i=0;i<count;i++) {
    const SpectreTouchKey *k=&keys[i];
    if (k->command && x>=k->x && x<k->x+k->w && y>=k->y && y<k->y+k->h)
      return (int)i;
  }
  int best=-1;double score=-INFINITY;
  for (size_t i=0;i<count;i++) {
    const SpectreTouchKey *k=&keys[i];
    if (k->command || fabs(y-(k->y+k->h/2))>k->h ||
        fabs(x-(k->x+k->w/2))>k->w*1.5) continue;
    double value=spectre_touch_score(k,x,y,prefix,adaptive);
    if (value>score) {score=value;best=(int)i;}
  }
  return best;
}

/* The prototype's erase() rule, expressed as a Unicode-codepoint boundary. */
static inline int spectre_grapheme_extension(unsigned int c) {
  return c==0xfe0f || c==0xfe0e || c==0x20e3 ||
    (c>=0x1f3fb && c<=0x1f3ff) || (c>=0x300 && c<=0x36f) ||
    (c>=0x64b && c<=0x65f) || c==0x670;
}
static inline size_t spectre_erase_start(const unsigned int *points,size_t length) {
  if (!length) return 0;
  size_t n=length-1;
  if (points[n]>=0x1f1e6 && points[n]<=0x1f1ff) {
    size_t count=0,j=length;
    while (j && points[j-1]>=0x1f1e6 && points[j-1]<=0x1f1ff) {count++;j--;}
    if (count%2==0 && n) n--;
  } else {
    while (n && spectre_grapheme_extension(points[n])) n--;
    while (n>1 && points[n-1]==0x200d) {
      n-=2;
      while (n && spectre_grapheme_extension(points[n])) n--;
    }
  }
  return n;
}
