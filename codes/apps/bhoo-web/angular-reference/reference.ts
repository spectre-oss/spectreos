// SpectreOS comparison fixtures. The controls themselves are unmodified Angular Material.
import '@angular/compiler';
import {Component, ChangeDetectorRef, inject, provideZonelessChangeDetection, enableProdMode} from '@angular/core';
import {bootstrapApplication} from '@angular/platform-browser';
import {CommonModule} from '@angular/common';
import {FormsModule} from '@angular/forms';
import {MatButtonModule} from '@angular/material/button';
import {MatCheckboxModule} from '@angular/material/checkbox';
import {MatSlideToggleModule} from '@angular/material/slide-toggle';
import {MatRadioModule} from '@angular/material/radio';
import {MatSliderModule} from '@angular/material/slider';
import {MatTabsModule} from '@angular/material/tabs';
import {MatChipsModule} from '@angular/material/chips';
import {MatListModule} from '@angular/material/list';
import {MatProgressBarModule} from '@angular/material/progress-bar';
import {MatProgressSpinnerModule} from '@angular/material/progress-spinner';
import {MatTooltipModule} from '@angular/material/tooltip';
import {MatBadgeModule} from '@angular/material/badge';
import {MatDividerModule} from '@angular/material/divider';
import {MatButtonToggleModule} from '@angular/material/button-toggle';
import {MatInputModule} from '@angular/material/input';
import {MatFormFieldModule} from '@angular/material/form-field';
import {MatSelectModule} from '@angular/material/select';
import {MatAutocompleteModule} from '@angular/material/autocomplete';
import {MatDatepickerModule} from '@angular/material/datepicker';
import {provideNativeDateAdapter} from '@angular/material/core';
import {MatTimepickerModule} from '@angular/material/timepicker';
import {MatToolbarModule} from '@angular/material/toolbar';
import {MatMenuModule} from '@angular/material/menu';
import {MatCardModule} from '@angular/material/card';
import {MatDialog, MatDialogModule} from '@angular/material/dialog';
import {MatBottomSheet, MatBottomSheetModule} from '@angular/material/bottom-sheet';
import {MatExpansionModule} from '@angular/material/expansion';
import {MatSnackBar, MatSnackBarModule} from '@angular/material/snack-bar';
import {MatTableModule} from '@angular/material/table';
import {MatSortModule} from '@angular/material/sort';
import {MatPaginatorModule} from '@angular/material/paginator';
import {MatStepperModule} from '@angular/material/stepper';
import {MatGridListModule} from '@angular/material/grid-list';
import {MatTreeModule} from '@angular/material/tree';
import {MatSidenavModule} from '@angular/material/sidenav';
import {MatRippleModule, MAT_RIPPLE_GLOBAL_OPTIONS} from '@angular/material/core';
import {ScrollingModule} from '@angular/cdk/scrolling';
import {OverlayContainer} from '@angular/cdk/overlay';

const imports = [CommonModule, FormsModule, MatButtonModule, MatCheckboxModule, MatSlideToggleModule,
  MatRadioModule, MatSliderModule, MatTabsModule, MatChipsModule, MatListModule, MatProgressBarModule,
  MatProgressSpinnerModule, MatTooltipModule, MatBadgeModule, MatDividerModule, MatButtonToggleModule,
  MatInputModule, MatFormFieldModule, MatSelectModule, MatAutocompleteModule, MatDatepickerModule,
  MatTimepickerModule, MatToolbarModule, MatMenuModule, MatCardModule, MatDialogModule,
  MatBottomSheetModule, MatExpansionModule, MatSnackBarModule, MatTableModule, MatSortModule,
  MatPaginatorModule, MatStepperModule, MatGridListModule, MatTreeModule, MatSidenavModule,
  MatRippleModule, ScrollingModule];

const templates: Record<string,string> = {
  button: `@for (item of buttons; track item[0]) {
    <button [matButton]="item[1]" [disabled]="disabled" (click)="count=count+1">{{item[0]}}</button>
  }<button [matButton]="variant" [disabled]="disabled" (click)="count=count+1">Your chosen variant</button>
  <p>Activations: {{count}}</p><button matButton="filled" disabled>Disabled button</button>`,
  'icon-button': `<p>Presentation glyph / separate accessible name</p>
    <button mat-icon-button aria-label="Add item" [disabled]="disabled" (click)="count=count+1">+</button>
    <button mat-icon-button aria-label="Close" [disabled]="disabled">×</button><p>Items added: {{count}}</p>`,
  checkbox: `<mat-checkbox [disabled]="disabled" [(ngModel)]="binary">Binary selection</mat-checkbox>
    <mat-checkbox [disabled]="disabled" [checked]="tri===1" [indeterminate]="tri===2" (change)="tri=(tri+1)%3">Unchecked / checked / mixed</mat-checkbox>
    <mat-checkbox disabled [indeterminate]="true">Disabled mixed state</mat-checkbox>
    <p>Tri-state value: {{['Unchecked','Checked','Mixed'][tri]}}</p>
    <button matButton [disabled]="disabled" (click)="tri=tri===2?1:2">Toggle mixed / checked</button>`,
  switch: `<mat-slide-toggle [disabled]="disabled" [(ngModel)]="notifications">Notifications</mat-slide-toggle>
    <mat-slide-toggle [disabled]="disabled" [(ngModel)]="updates">Background updates</mat-slide-toggle>
    <mat-slide-toggle disabled [checked]="true">Disabled switch</mat-slide-toggle><p>Notifications: {{notifications}}</p>`,
  'radio-group': `<p>Power mode</p><mat-radio-group aria-label="Power mode" [disabled]="disabled" [(ngModel)]="selected">
    <mat-radio-button [value]="0">Balanced</mat-radio-button><mat-radio-button [value]="1">Performance</mat-radio-button>
    <mat-radio-button [value]="2" disabled>Unavailable</mat-radio-button></mat-radio-group><p>Selected option: {{selected+1}}</p>`,
  slider: `<p>Volume: {{value}}%</p><mat-slider [disabled]="disabled" min="0" max="100" step="1"><input aria-label="Volume" matSliderThumb [(ngModel)]="value"></mat-slider>
    <mat-slider disabled min="0" max="100" step="5"><input aria-label="Disabled slider" matSliderThumb [value]="65"></mat-slider>`,
  'range-slider': `<p>Range: {{start}} – {{end}}</p><mat-slider [disabled]="disabled" min="0" max="100" step="1">
    <input matSliderStartThumb aria-label="Range start" [(ngModel)]="start"><input matSliderEndThumb aria-label="Range end" [(ngModel)]="end"></mat-slider>
    <p>Tab between endpoints. Use arrows or drag.</p>`,
  tabs: `<mat-tab-group [animationDuration]="reduced?'0ms':'500ms'" [(selectedIndex)]="selected">@for (name of tabNames; track name) {
    <mat-tab [label]="name" [disabled]="disabled"></mat-tab>}</mat-tab-group><p>{{tabNames[selected]}} selected</p>`,
  chip: `<div class="row chip-row"><mat-chip-listbox multiple aria-label="Filters"><mat-chip-option style="width:100%;margin:4px 0" [disabled]="disabled" [selected]="binary" (selectionChange)="binary=$event.selected">Filter: installed</mat-chip-option></mat-chip-listbox>
    <mat-chip-listbox aria-label="Unavailable filter"><mat-chip-option style="width:100%;margin:4px 0" disabled>Disabled chip</mat-chip-option></mat-chip-listbox></div><p>Filter selected: {{binary}}</p>`,
  'list-and-list-item': `<mat-selection-list [disabled]="disabled" [multiple]="false">@for (name of ['Applications','Appearance','Keyboard','Network']; track name; let i=$index) {
    <mat-list-option [selected]="selected===i" (click)="selected=i">{{name}}</mat-list-option>}</mat-selection-list>`,
  'virtualized-list': `<p>100,000 entries. Only visible rows are built.</p><cdk-virtual-scroll-viewport itemSize="40" class="scroll-list" (scrolledIndexChange)="offset=$event*40">
    <mat-selection-list [disabled]="disabled" [multiple]="false"><mat-list-option style="height:40px" *cdkVirtualFor="let item of items" (click)="listSelected=item+1">Application</mat-list-option></mat-selection-list></cdk-virtual-scroll-viewport><p>Selected: {{listSelected}} / offset: {{offset}}px</p>`,
  'progress-indicator': `<p>Determinate / value-driven</p><mat-slider [disabled]="disabled" min="0" max="100"><input aria-label="Progress" matSliderThumb [(ngModel)]="value"></mat-slider>
    <mat-progress-bar mode="determinate" [value]="value" aria-label="Download"></mat-progress-bar><p>Indeterminate / visible-only motion</p>
    <mat-progress-bar [mode]="reduced?'determinate':'indeterminate'" [value]="40" aria-label="Working"></mat-progress-bar>
    <p>Circular / determinate and indeterminate</p><div class="row"><mat-progress-spinner mode="determinate" [value]="value" diameter="56" aria-label="Download"></mat-progress-spinner>
    <mat-progress-spinner [mode]="reduced?'determinate':'indeterminate'" [value]="40" diameter="56" aria-label="Working"></mat-progress-spinner></div>`,
  tooltip: `<p>Hover for half a second, or focus with Tab.</p><button matButton="outlined" [disabled]="disabled" matTooltip="A small, useful detail" [matTooltipShowDelay]="500">Hover or focus me</button>`,
  badge: `<p>Count / label badges</p>@for (badge of ['3','12','New']; track badge) {<div class="badge-row"><span class="badge-host" [matBadge]="badge" [matBadgeDisabled]="disabled" matBadgeOverlap="false" [attr.aria-label]="badge"></span></div>}`,
  divider: `<p>Before the divider</p><mat-divider></mat-divider><p>After the divider</p>`,
  'focus-ring': `<button matButton="filled" [disabled]="disabled">First action</button><button matButton="filled" [disabled]="disabled">Second action</button>`,
  'typography-text': `<p class="mat-body-large">Bhoo / interface text</p><p class="mat-body-medium">System typography and Unicode: SpectreOS — روح</p>`,
  'floating-action-button': `<p>Create — small, regular and extended</p><button mat-mini-fab [disabled]="disabled" aria-label="Create small" (click)="count=count+1">+</button>
    <button mat-fab [disabled]="disabled" aria-label="Create regular" (click)="count=count+1">+</button><button mat-fab extended [disabled]="disabled" (click)="count=count+1">+ Create item</button><p>Created: {{count}}</p>`,
  'segmented-selection-toggle-group': `<p>Single selection</p><mat-button-toggle-group [disabled]="disabled" [(ngModel)]="selected" aria-label="View">
    @for (name of ['Day','Week','Month']; track name; let i=$index) {<mat-button-toggle [value]="i">{{name}}</mat-button-toggle>}</mat-button-toggle-group>
    <p>Multiple selection</p><mat-button-toggle-group multiple [disabled]="disabled" aria-label="Formatting"><mat-button-toggle value="bold">Bold</mat-button-toggle><mat-button-toggle value="italic">Italic</mat-button-toggle><mat-button-toggle value="mono">Mono</mat-button-toggle></mat-button-toggle-group>`,
  'button-group': `<p>Related actions with independent outcomes</p><div class="row">@for (name of ['Cut','Copy','Paste']; track name) {<button matButton [disabled]="disabled" (click)="notice=name">{{name}}</button>}</div><p>{{notice}}</p>`,
  'text-field': `<p>Text field</p><mat-form-field appearance="outline"><mat-label>Name</mat-label><input matInput [disabled]="disabled" [(ngModel)]="text"></mat-form-field><p>{{text.length}} characters</p>`,
  'multiline-text-field': `<p>Multiline text field</p><mat-form-field appearance="outline"><mat-label>Notes</mat-label><textarea matInput [disabled]="disabled" [(ngModel)]="text" rows="3"></textarea></mat-form-field><p>{{text.length}} characters</p>`,
  search: `<mat-form-field appearance="outline"><mat-label>Search applications</mat-label><input matInput type="search" [disabled]="disabled" [(ngModel)]="text"></mat-form-field><mat-list>@for (name of suggestions; track name) {<mat-list-item>{{name}}</mat-list-item>}</mat-list>`,
  select: `<p>Power profile</p><mat-form-field appearance="outline"><mat-label>Power profile</mat-label><mat-select [disabled]="disabled" [(ngModel)]="selected">@for (name of ['Balanced','Performance','Battery saver']; track name; let i=$index) {<mat-option [value]="i">{{name}}</mat-option>}</mat-select></mat-form-field><p>Arrow keys move through items; Escape dismisses.</p>`,
  autocomplete: `<mat-form-field appearance="outline"><mat-label>Search applications</mat-label><input matInput [disabled]="disabled" [(ngModel)]="text" [matAutocomplete]="auto"><mat-autocomplete #auto="matAutocomplete">@for (name of suggestions; track name) {<mat-option [value]="name">{{name}}</mat-option>}</mat-autocomplete></mat-form-field>`,
  'form-field-container': `<mat-form-field appearance="outline"><mat-label>Required name</mat-label><input matInput required [disabled]="disabled" [(ngModel)]="text"><mat-hint>Enter your name</mat-hint><mat-error>Name is required</mat-error></mat-form-field>`,
  'date-picker': `<mat-calendar [startAt]="date" [(selected)]="date" [dateFilter]="dateEnabled"></mat-calendar>`,
  'date-range-picker': `<mat-form-field><mat-label>Date range</mat-label><mat-date-range-input [rangePicker]="range" [disabled]="disabled"><input matStartDate placeholder="Start date"><input matEndDate placeholder="End date"></mat-date-range-input><mat-datepicker-toggle matIconSuffix [for]="range"></mat-datepicker-toggle><mat-date-range-picker #range></mat-date-range-picker></mat-form-field>`,
  'time-picker': `<mat-form-field><mat-label>Select time</mat-label><input matInput [matTimepicker]="time" [disabled]="disabled"><mat-timepicker-toggle matIconSuffix [for]="time"></mat-timepicker-toggle><mat-timepicker #time></mat-timepicker></mat-form-field>`,
  toolbar: `<p>Application actions</p><mat-toolbar>@for (name of ['Menu','Search','Share']; track name) {<button matButton [disabled]="disabled" (click)="notice=name+' requested'">{{name}}</button>}</mat-toolbar><p>{{notice}}</p>`,
  menu: `<button matButton [matMenuTriggerFor]="menu" [disabled]="disabled">Open anchored surface</button><p>{{notice}}</p><mat-menu #menu="matMenu">@for (name of ['Open','Rename','Remove']; track name) {<button mat-menu-item [disabled]="disabled" (click)="notice='Selected: '+name">{{name}}</button>}</mat-menu>`,
  card: `@for (name of ['Filled card','Outlined card','Elevated card']; track name; let i=$index) {<mat-card [appearance]="i===0?'filled':i===1?'outlined':'raised'"><button matButton [disabled]="disabled" (click)="selected=i">{{name}}</button></mat-card>}<p>Selected card: {{selected+1}}</p>`,
  dialog: `<button matButton="filled" [disabled]="disabled" (click)="openDialog(dialogContent)">Open surface</button><p>{{notice}}</p><ng-template #dialogContent><h2 mat-dialog-title>Save changes?</h2><mat-dialog-content><p>Your workspace has unsaved changes.</p><p>Escape or the scrim dismisses this surface.</p></mat-dialog-content><mat-dialog-actions><button matButton mat-dialog-close="Cancelled">Cancel</button><button matButton mat-dialog-close="Saved">Save</button></mat-dialog-actions></ng-template>`,
  'bottom-sheet': `<button matButton="filled" [disabled]="disabled" (click)="openSheet(sheetContent)">Open surface</button><p>{{notice}}</p><ng-template #sheetContent><h2>Save changes?</h2><p>Your workspace has unsaved changes.</p><p>Escape or the scrim dismisses this surface.</p><button matButton (click)="closeSheet('Cancelled')">Cancel</button><button matButton (click)="closeSheet('Saved')">Save</button></ng-template>`,
  'expansion-panel-accordion': `<mat-accordion multi>@for (name of ['Section one','Section two','Section three']; track name) {<mat-expansion-panel [disabled]="disabled"><mat-expansion-panel-header><mat-panel-title>{{name}}</mat-panel-title></mat-expansion-panel-header><mat-list><mat-list-item>Child one</mat-list-item><mat-list-item>Child two</mat-list-item></mat-list></mat-expansion-panel>}</mat-accordion>`,
  snackbar: `<button matButton="filled" [disabled]="disabled" (click)="archive()">Archive message</button><p>{{notice}}</p>`,
  'data-table': `<table mat-table [dataSource]="rows" matSort (matSortChange)="sort($event)"><ng-container matColumnDef="name"><th mat-header-cell *matHeaderCellDef mat-sort-header [disabled]="disabled" style="width:55%">Name</th><td mat-cell *matCellDef="let row">{{row.name}}</td></ng-container><ng-container matColumnDef="size"><th mat-header-cell *matHeaderCellDef mat-sort-header [disabled]="disabled">Size</th><td mat-cell *matCellDef="let row">{{row.size}}</td></ng-container><tr mat-header-row *matHeaderRowDef="['name','size']"></tr><tr mat-row *matRowDef="let row; columns: ['name','size']"></tr></table>`,
  paginator: `<mat-paginator [length]="95" [pageSize]="10" [pageIndex]="selected" [disabled]="disabled" (page)="selected=$event.pageIndex" aria-label="Results"></mat-paginator>@for (i of [1,2,3]; track i) {<p>Result {{selected*10+i}}</p>}`,
  stepper: `<mat-stepper linear [animationDuration]="reduced?'0ms':'500ms'"><ng-template matStepperIcon="edit"><svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true"><path fill="currentColor" d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04a1 1 0 0 0 0-1.41l-2.34-2.34a1 1 0 0 0-1.41 0l-1.83 1.83 3.75 3.75z"/></svg></ng-template><mat-step label="Your name" [completed]="!!text.trim()"><div class="step-content"><mat-form-field><mat-label>Name</mat-label><input matInput [disabled]="disabled" [(ngModel)]="text"></mat-form-field><button matButton matStepperNext [disabled]="disabled||!text.trim()">Next</button></div></mat-step><mat-step label="Profile"><div class="step-content"><p>Choose your profile</p><div class="row"><button matButton matStepperPrevious [disabled]="disabled">Back</button><button matButton matStepperNext [disabled]="disabled">Next</button></div></div></mat-step><mat-step label="Done"><div class="step-content"><p>Ready</p><button matButton matStepperPrevious [disabled]="disabled">Back</button></div></mat-step></mat-stepper>`,
  'grid-list-image-list': `<mat-grid-list cols="2" rowHeight="120px" gutterSize="12px">@for (name of ['Files','Browser','Music','Settings']; track name) {<mat-grid-tile><button matButton="tonal" [disabled]="disabled">{{name}}</button></mat-grid-tile>}</mat-grid-list>`,
  tree: `<mat-tree [dataSource]="treeData" [childrenAccessor]="childrenAccessor"><mat-tree-node *matTreeNodeDef="let node" matTreeNodePadding>{{node.name}}</mat-tree-node><mat-tree-node *matTreeNodeDef="let node; when: hasChild" matTreeNodePadding matTreeNodeToggle [isExpandable]="true" [isDisabled]="disabled"><button mat-icon-button matTreeNodeToggle [disabled]="disabled" aria-label="Toggle section">›</button>{{node.name}}</mat-tree-node></mat-tree>`,
  'navigation-drawer': `<mat-sidenav-container style="height:350px"><mat-sidenav mode="side" opened style="width:176px"><mat-nav-list>@for (name of ['Home','Files','Settings']; track name; let i=$index) {<a mat-list-item [activated]="selected===i" [disabled]="disabled" (click)="selected=i">{{name}}</a>}</mat-nav-list></mat-sidenav><mat-sidenav-content style="padding:24px">{{['Home — overview','Files — your documents','Settings — preferences'][selected]}}</mat-sidenav-content></mat-sidenav-container>`,
  'state-layer-ripple': `<p>Hover, focus and press feedback</p><button matButton="filled" [disabled]="disabled" (click)="count=count+1">Press and hold</button><p>Activations: {{count}}</p><p>Pointer-origin ink • hold to expand • release to fade.</p>`,
};

class Reference {
  cdr = inject(ChangeDetectorRef);
  dialog = inject(MatDialog);
  sheets = inject(MatBottomSheet);
  snacks = inject(MatSnackBar);
  ripple = inject(MAT_RIPPLE_GLOBAL_OPTIONS);
  overlays = inject(OverlayContainer);
  disabled = false; reduced = false; dark = false; variant = 'filled'; key = 'button';
  offset=0;listSelected=0;
  count=0; binary=false; tri=2; notifications=true; updates=false; selected=0;
  value=40; start=25; end=75; text=''; notice=''; date=new Date(2026,10,9);
  buttons=[['Filled button','filled'],['Tonal button','tonal'],['Outlined button','outlined'],['Text button','text'],['Elevated color treatment','elevated']];
  tabNames=['Overview','Activity','Settings'];
  items=Array.from({length:100_000},(_,i)=>i);
  rows=[{name:'Bhoo',size:86},{name:'Launcher',size:240},{name:'SpectreOS',size:120}];
  treeData=[{name:'Section one',children:[{name:'Child one'},{name:'Child two'}]},{name:'Section two',children:[{name:'Child one'},{name:'Child two'}]}];
  childrenAccessor=(node:any)=>node.children??[];
  hasChild=(_:number,node:any)=>!!node.children?.length;
  dateEnabled=()=>!this.disabled;
  get suggestions(){return ['Bhoo','Browser','Files','Settings','Terminal'].filter(n=>n.toLowerCase().includes(this.text.toLowerCase()));}
  openDialog(template:any){this.dialog.open(template,{width:'400px',...(this.reduced?{enterAnimationDuration:0,exitAnimationDuration:0}:{})}).afterClosed().subscribe(result=>{this.notice=result??'';this.cdr.markForCheck();});}
  openSheet(template:any){this.sheets.open(template);}
  closeSheet(result:string){this.notice=result;this.sheets.dismiss();}
  archive(){const ref=this.snacks.open('Message archived','Undo',{duration:6000});this.notice='Message archived';ref.onAction().subscribe(()=>{this.notice='Archive undone';this.cdr.markForCheck();});}
  sort(event:any){const sign=event.direction==='desc'?-1:1;this.rows=[...this.rows].sort((a:any,b:any)=>sign*(typeof a[event.active]==='number'?a[event.active]-b[event.active]:a[event.active].localeCompare(b[event.active])));}
}

const variants=['text','filled','tonal','outlined','text','elevated'];
const query=new URLSearchParams(location.search);
const key=query.get('fixture')||'button';
const exists=Object.hasOwn(templates,key);
const template=exists?'<div class="fixture">'+templates[key]+'</div>':'<div class="empty"><h2>No direct Angular Material component</h2><p>This Bhoo family has no equivalent in the pinned Angular Material package. Use its own specification and reference implementations.</p></div>';
Component({selector:'angular-reference',standalone:true,imports,template})(Reference);
enableProdMode();
const debug:any=(window as any).angularReference={ready:false,key,exists,version:'22.2.6',errors:[],settings:{},template:templates[key]||''};
const run=Number(query.get('run'));
const send=(type:string,data:any={})=>parent.postMessage({source:'bhoo-angular',type,run,...data},location.origin==='null'?'*':location.origin);
window.addEventListener('error',event=>{debug.errors.push(event.message);send('error',{message:event.message});});
window.addEventListener('unhandledrejection',event=>{debug.errors.push(String(event.reason));send('error',{message:String(event.reason)});});
bootstrapApplication(Reference,{providers:[provideZonelessChangeDetection(),provideNativeDateAdapter(),{provide:MAT_RIPPLE_GLOBAL_OPTIONS,useValue:{}}]}).then(app=>{
  const ref=app.components[0]; const instance=ref.instance as Reference; instance.key=key;
  // Register the initial indeterminate state after the checkbox view exists,
  // so Angular can select its mixed-to-checked animation on the first toggle.
  if (key === 'checkbox') {
    instance.tri = 0; instance.cdr.markForCheck(); instance.cdr.detectChanges();
    instance.tri = 2; instance.cdr.markForCheck(); instance.cdr.detectChanges();
  }
  const palette=()=>{
    const style=getComputedStyle(document.documentElement);
    const names=['surface','on-surface','surface-container','surface-container-high','surface-container-highest','primary','on-primary','secondary-container','outline','error','on-secondary-container','surface-container-low','outline-variant','primary-container','on-primary-container','inverse-surface','inverse-on-surface','on-surface-variant','surface-variant','inverse-primary','tertiary-container','on-tertiary-container'];
    const probe=document.createElement('span');document.body.append(probe);
    const colors=names.map(name=>{probe.style.color=style.getPropertyValue('--mat-sys-'+name).trim();const color=getComputedStyle(probe).color;const c=document.createElement('canvas').getContext('2d')!;c.fillStyle=color;c.fillRect(0,0,1,1);return [...c.getImageData(0,0,1,1).data];});probe.remove();return colors;
  };
  const sendCheckboxState=()=>{if(key!=='checkbox')return;const inputs=[...document.querySelectorAll<HTMLInputElement>('mat-checkbox input')];const focused=inputs.indexOf(document.activeElement as HTMLInputElement)+1;send('checkbox-state',{state:{binary:instance.binary,tri:instance.tri,focused}});};
  for(const type of ['change','click','focusin','focusout'])document.addEventListener(type,()=>requestAnimationFrame(sendCheckboxState),true);
  window.addEventListener('message',event=>{
    if(event.origin!==location.origin||event.source!==parent||event.data?.source!=='bhoo-comparison'||event.data.run!==run)return;
    if(event.data.type==='checkbox-state-request')sendCheckboxState();
    if(event.data.type==='visibility')document.documentElement.dataset.paused=String(!event.data.visible);
    if(event.data.type==='settings'){
      const s=event.data.settings;instance.disabled=!!s.disabled;instance.reduced=!!s.reduced;instance.dark=!!s.dark;instance.variant=variants[s.variant]||'filled';
      document.documentElement.dataset.theme=instance.dark?'dark':'light';document.documentElement.dataset.reduced=String(instance.reduced);
      instance.ripple.disabled=instance.reduced;
      instance.overlays.getContainerElement().style.colorScheme=instance.dark?'dark':'light';
      if(instance.disabled){instance.dialog.closeAll();instance.sheets.dismiss();instance.snacks.dismiss();}
      debug.settings={...s};instance.cdr.markForCheck();instance.cdr.detectChanges();send('palette',{colors:palette()});
    }
    if(event.data.type==='activate'){
      const el=[...document.querySelectorAll<HTMLElement>('button:not(:disabled),input:not(:disabled),[role="option"],[role="tab"]')].find(el=>el.getBoundingClientRect().width>0);
      el?.focus();el?.click();instance.cdr.markForCheck();instance.cdr.detectChanges();
    }
  });
  debug.ready=true;send('ready',{version:debug.version,key,exists,colors:palette()});
}).catch(error=>{debug.errors.push(String(error));document.querySelector('angular-reference')!.textContent='Angular reference failed: '+error;send('error',{message:String(error)});});
