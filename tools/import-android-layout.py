#!/usr/bin/env python3
"""Compile an explicit Android XML subset to static Bhoo nodes; no Android runtime.
Unsupported tags/attributes/resources fail rather than silently changing layout.
"""
import argparse,json,math,re,datetime,tempfile,xml.etree.ElementTree as ET
from pathlib import Path
from android_png import decode_png, decode_nine_patch, decode_nine_patch_full, decode_nine_patch_bands, is_nine_patch
from android_vector import rasterize_vector, vector_tint, vector_tint_mode, vector_auto_mirrored
A='{http://schemas.android.com/apk/res/android}'
APP='{http://schemas.android.com/apk/res-auto}'
TOOLS='{http://schemas.android.com/tools}'
AAPT='{http://schemas.android.com/aapt}'
DENSITY_QUALIFIERS={'ldpi':120,'mdpi':160,'hdpi':240,'xhdpi':320,'xxhdpi':480,'xxxhdpi':640,'nodpi':0,'anydpi':-1}
RTL_LANGUAGES={'ar','he','iw','fa','ur'}
LOCALE_LANG_CANON={'iw':'he','in':'id','ji':'yi'}
VALID_INPUT_TYPE_FLAGS={
    'none','text','textCapCharacters','textCapWords','textCapSentences',
    'textAutoCorrect','textAutoComplete','textMultiLine','textImeMultiLine',
    'textNoSuggestions','textUri','textEmailAddress','textEmailSubject',
    'textShortMessage','textLongMessage','textPersonName','textPostalAddress',
    'textPassword','textVisiblePassword','textWebEditText','textFilter',
    'textPhonetic','textWebEmailAddress','textWebPassword',
    'number','numberSigned','numberDecimal','numberPassword',
    'phone','datetime','date','time',
}
CONTAINER_TAGS={'DialerFilter','TabHost','TabWidget','SlidingDrawer','Toolbar','GridLayout','TwoLineListItem','TableLayout','TableRow','LinearLayout','RadioGroup','FrameLayout','MediaController','RelativeLayout','ConstraintLayout','AbsoluteLayout','ViewAnimator','ViewSwitcher','TextSwitcher','ImageSwitcher','ViewFlipper','ScrollView','HorizontalScrollView','com.google.android.material.card.MaterialCardView'}
MATERIAL_DIVIDER_ATTRS={'dividerThickness','dividerColor','dividerInsetStart','dividerInsetEnd'}
MATERIAL_CARD_ATTRS={'checkedIconSize','checkedIconMargin','checkedIconGravity','checkedIconTint','cardBackgroundColor','cardForegroundColor','strokeColor','strokeWidth','rippleColor','cardCornerRadius','cardElevation','cardMaxElevation','cardUseCompatPadding','cardPreventCornerOverlap','contentPadding','contentPaddingLeft','contentPaddingTop','contentPaddingRight','contentPaddingBottom'}
CARD_THEME_COLORS={'colorPrimary','colorOnPrimary','colorSecondary','colorOnSecondary','colorSurface','colorOnSurface','colorOutline','colorOutlineVariant','colorOnSurfaceVariant','colorSurfaceContainerLow','colorSurfaceContainerHigh','colorSurfaceContainerHighest'}
APP_DRAWABLE_COMPAT_ATTRS={'drawableStartCompat','drawableEndCompat','drawableTopCompat','drawableBottomCompat','drawableLeftCompat','drawableRightCompat','drawableTint','drawableTintMode'}
APP_BUTTON_ATTRS={'cornerRadius','strokeColor','strokeWidth','icon','iconTint','iconSize','iconPadding','iconGravity','rippleColor','shapeAppearance','shapeAppearanceOverlay'}
TEXT_INPUT_ATTRS={'hintEnabled','errorEnabled','boxBackgroundMode','boxBackgroundColor','boxCornerRadiusTopStart','boxCornerRadiusTopEnd','boxCornerRadiusBottomStart','boxCornerRadiusBottomEnd','boxStrokeColor','boxStrokeWidth','boxStrokeWidthFocused','endIconMode','endIconDrawable','endIconTint','passwordToggleEnabled','helperText','helperTextEnabled','counterEnabled','counterMaxLength','startIconDrawable','startIconTint','hintTextAppearance','hintTextColor','placeholderText','prefixText','suffixText','expandedHintEnabled'}
CHIP_ATTRS={'chipBackgroundColor','chipStrokeColor','chipStrokeWidth','chipCornerRadius','chipMinHeight','chipIcon','chipIconVisible','chipIconSize','chipIconTint','closeIcon','closeIconVisible','closeIconTint','checkedIcon','checkedIconVisible','checkedIconTint','checkable','ensureMinTouchTargetSize','chipStartPadding','chipEndPadding','textStartPadding','textEndPadding'}
NAV_STYLE_ATTRS={'labelVisibilityMode','menuGravity','headerLayout'}
TOGGLE_GROUP_ATTRS={'singleSelection','selectionRequired','checkedButton'}
EXTENDED_FAB_ATTRS={'extended','extendMotionSpec','shrinkMotionSpec','showMotionSpec','hideMotionSpec','collapsedSize','extendStrategy','elevation','ensureMinTouchTargetSize'}
BOTTOM_APP_BAR_ATTRS={'menu','fabAlignmentMode','fabAnimationMode','fabAnchorMode','menuAlignmentMode','fabCradleMargin','fabCradleRoundedCornerRadius','fabCradleVerticalOffset','fabAlignmentModeEndMargin','removeEmbeddedFabElevation','addElevationShadow','hideOnScroll','paddingBottomSystemWindowInsets','paddingLeftSystemWindowInsets','paddingRightSystemWindowInsets'}
APP_BAR_ATTRS={'expanded','liftOnScroll','liftOnScrollTargetViewId','liftOnScrollColor','statusBarForeground','lifted'}
COLLAPSING_TOOLBAR_ATTRS={'title','subtitle','titleEnabled','contentScrim','statusBarScrim','toolbarId','collapsedTitleGravity','expandedTitleGravity','expandedTitleMargin','expandedTitleMarginStart','expandedTitleMarginEnd','expandedTitleMarginTop','expandedTitleMarginBottom','expandedTitleSpacing','collapsedTitleTextAppearance','expandedTitleTextAppearance','collapsedSubtitleTextAppearance','expandedSubtitleTextAppearance','collapsedTitleTextColor','expandedTitleTextColor','collapsedSubtitleTextColor','expandedSubtitleTextColor','scrimAnimationDuration','scrimVisibleHeightTrigger','titleCollapseMode','titleTextEllipsize','maxLines','titleMaxLines','subtitleMaxLines','forceApplySystemWindowInsetTop','extraMultilineHeightEnabled','titlePositionInterpolator','collapsedTitleGravityMode'}
SLIDER_ATTRS={'values','minSeparation','minSeparationUnit','thumbColor','thumbRadius','thumbWidth','thumbHeight','thumbElevation','thumbStrokeColor','thumbStrokeWidth','trackColor','trackColorActive','trackColorInactive','trackHeight','tickColor','tickColorActive','tickColorInactive','tickVisible','tickVisibilityMode','tickRadiusActive','tickRadiusInactive','haloColor','haloRadius','labelBehavior','labelStyle','trackStopIndicatorSize','thumbTrackGapSize','centered','continuousModeTickCount','minTouchTargetSize','trackIconActiveStart','trackIconActiveEnd','trackIconActiveColor','trackIconInactiveStart','trackIconInactiveEnd','trackIconInactiveColor','trackIconSize','trackCornerSize','trackInsideCornerSize'}
LOADING_INDICATOR_ATTRS={'indicatorColor','containerColor','indicatorSize','containerWidth','containerHeight','showDelay','minHideDelay'}
PROGRESS_INDICATOR_ATTRS={'indeterminate','indicatorColor','trackColor','trackThickness','trackCornerRadius','indicatorSize','indicatorInset','showAnimationBehavior','hideAnimationBehavior','indeterminateAnimationType','indeterminateAnimationTypeCircular','indicatorDirectionLinear','indicatorDirectionCircular','minHideDelay','showDelay','indicatorTrackGapSize','wavelength','wavelengthDeterminate','wavelengthIndeterminate','waveAmplitude','waveSpeed','trackStopIndicatorSize','trackInnerCornerRadius','trackStopIndicatorPadding','indeterminateTrackVisible','indeterminateAnimatorDurationScale','waveAmplitudeRampProgressMin','waveAmplitudeRampProgressMax'}
CHIP_GROUP_ATTRS={'singleLine','singleSelection','selectionRequired','chipSpacing','chipSpacingHorizontal','chipSpacingVertical','checkedChip'}
TAB_LAYOUT_ATTRS={'tabMode','tabGravity','tabIndicator','tabIndicatorColor','tabIndicatorHeight','tabIndicatorFullWidth','tabIndicatorGravity','tabIndicatorAnimationMode','tabIndicatorAnimationDuration','tabTextColor','tabSelectedTextColor','tabTextAppearance','tabSelectedTextAppearance','tabBackground','tabPadding','tabPaddingStart','tabPaddingEnd','tabPaddingTop','tabPaddingBottom','tabMaxWidth','tabMinWidth','tabContentStart','tabInlineLabel','tabIconTint','tabIconTintMode','tabRippleColor','tabUnboundedRipple'}
ALERT_DIALOG_ATTRS={'backgroundInsetStart','backgroundInsetTop','backgroundInsetEnd','backgroundInsetBottom','icon'}
SNACKBAR_ATTRS={'maxActionInlineWidth','animationMode','backgroundOverlayColorAlpha','actionTextColorAlpha','backgroundTintMode'}
FLOATING_TOOLBAR_ATTRS={'marginTopSystemWindowInsets','marginLeftSystemWindowInsets','marginRightSystemWindowInsets','marginBottomSystemWindowInsets'}
BUTTON_GROUP_ATTRS={'spacing','buttonSizeChange','innerCornerSize','overflowMode','overflowButtonIcon'}
FLOW_ATTRS={'flow_horizontalGap','flow_verticalGap','flow_wrapMode','flow_horizontalStyle','flow_verticalStyle','flow_firstHorizontalStyle','flow_firstVerticalStyle','flow_lastHorizontalStyle','flow_lastVerticalStyle','flow_horizontalBias','flow_verticalBias','flow_firstHorizontalBias','flow_firstVerticalBias','flow_lastHorizontalBias','flow_lastVerticalBias','flow_horizontalAlign','flow_verticalAlign','flow_maxElementsWrap','constraint_referenced_ids'}
RECYCLER_VIEW_ATTRS={'spanCount','reverseLayout','stackFromEnd','fastScrollEnabled'}
SEARCH_VIEW_ATTRS={'autoShowKeyboard','hideNavigationIcon','animateNavigationIcon','animateMenuItems','useDrawerArrowDrawable','headerLayout','backHandlingEnabled'}
SWIPE_REFRESH_ATTRS={'swipeRefreshLayoutProgressSpinnerBackgroundColor','swipeRefreshProgressColor','swipeRefreshSize','swipeRefreshOffsetStart','swipeRefreshOffsetEnd'}
CAROUSEL_ATTRS={'carousel_strategy','carousel_alignment','carousel_itemSpacing'}
VIEW_PAGER_ATTRS={'userInputEnabled','pageMargin','pageTransformer','offscreenPageLimit'}
TEXT_BEARING_TAGS={'TextView','CheckedTextView','Chronometer','TextClock','DigitalClock','Button','ToggleButton','CheckBox','RadioButton','Switch','EditText','AutoCompleteTextView','MultiAutoCompleteTextView'}
MATERIAL_ICON_NAMES={
    'widgets','code','palette','pause','dark_mode','light_mode','download','menu','menu_open',
    'vertical_align_top','vertical_align_bottom','close','check','add','arrow_back','arrow_forward',
    'search','more_vert','home','settings','inbox','star','archive','chevron_right','person','delete',
    'mark_email_unread','mail','chat_bubble','videocam','edit','sell','attach_file','send','reply',
    'more_horiz','chevron_left','image','sentiment_satisfied','description',
}
NAV_ICON_ALIASES={
    'chat':'chat_bubble','message':'chat_bubble','messages':'chat_bubble','forum':'chat_bubble',
    'email':'mail','drafts':'mail','all_inbox':'inbox',
    'account':'person','account_circle':'person','profile':'person','contacts':'person','people':'person','group':'person',
    'favorite':'star','favorites':'star','bookmark':'star','bookmarks':'star',
    'trash':'delete','bin':'delete','remove':'delete',
    'preferences':'settings','tune':'settings','cog':'settings',
    'video':'videocam','meet':'videocam','call':'videocam',
    'compose':'edit','create':'edit','pencil':'edit',
    'label':'sell','tag':'sell','tags':'sell','labels':'sell',
    'photo':'image','gallery':'image','photos':'image','picture':'image',
    'doc':'description','document':'description','article':'description','file':'description','notes':'description','note':'description','list':'description',
    'unread':'mark_email_unread','attach':'attach_file','attachment':'attach_file',
    'back':'arrow_back','forward':'arrow_forward','next':'chevron_right','prev':'chevron_left',
    'more':'more_vert','overflow':'more_vert','plus':'add','done':'check','clear':'close','cancel':'close',
    'mood':'sentiment_satisfied','emoji':'sentiment_satisfied',
    'theme':'palette','color':'palette','night':'dark_mode','day':'light_mode',
    'dashboard':'widgets','explore':'widgets','apps':'widgets','grid':'widgets',
}
COORDINATOR_ANCHOR_GRAVITIES={'top','bottom','start','end','left','right','center','center_horizontal','center_vertical'}
APP_LAYOUT_PARAMS_ATTRS={'layoutManager','layout_behavior','layout_scrollFlags','layout_scrollEffect','layout_scrollInterpolator','drawerLockMode','behavior_overlapTop','behavior_peekHeight','behavior_hideable','behavior_fitToContents','behavior_halfExpandedRatio','behavior_skipCollapsed','behavior_draggable','behavior_draggableOnNestedScroll','behavior_expandedOffset','behavior_significantVelocityThreshold','behavior_saveFlags','shouldRemoveExpandedCorners','gestureInsetBottomIgnored','paddingBottomSystemWindowInsets','paddingLeftSystemWindowInsets','paddingRightSystemWindowInsets','paddingTopSystemWindowInsets','marginLeftSystemWindowInsets','marginRightSystemWindowInsets','marginTopSystemWindowInsets','marginBottomSystemWindowInsets','coplanarSiblingViewId','layout_overflowIcon','layout_overflowText','layout_collapseMode','layout_collapseParallaxMultiplier','layout_anchor','layout_anchorGravity','layout_dodgeInsetEdges','layout_insetEdge','layout_constrainedWidth','layout_constrainedHeight','layout_constraintHorizontal_chainStyle','layout_constraintVertical_chainStyle','layout_constraintHorizontal_weight','layout_constraintVertical_weight','layout_constraintDimensionRatio','layout_constraintWidth_max','layout_constraintWidth_min','layout_constraintHeight_max','layout_constraintHeight_min','layout_constraintWidth_default','layout_constraintHeight_default','layout_constraintWidth_percent','layout_constraintHeight_percent','layout_constraintBaseline_toBaselineOf','layout_constraintBaseline_toTopOf','layout_constraintBaseline_toBottomOf','layout_goneMarginStart','layout_goneMarginEnd','layout_goneMarginLeft','layout_goneMarginRight','layout_goneMarginTop','layout_goneMarginBottom','layout_goneMarginBaseline','layout_constraintCircle','layout_constraintCircleAngle','layout_constraintCircleRadius','layout_constraintTag'}
SHAPE_APPEARANCE_KEYS={'cornerSize','cornerSizeTopLeft','cornerSizeTopRight','cornerSizeBottomRight','cornerSizeBottomLeft','cornerFamily','cornerFamilyTopLeft','cornerFamilyTopRight','cornerFamilyBottomRight','cornerFamilyBottomLeft'}
APP_AUTOSIZE_ATTRS={'autoSizeTextType','autoSizeMinTextSize','autoSizeMaxTextSize','autoSizeStepGranularity'}
BADGE_EXT_ATTRS={'badgeGravity','badgeMaxCharacterCount','badgeHorizontalOffset','badgeVerticalOffset'}
APP_COMMON_ATTRS={'srcCompat','tint','backgroundTint','fontFamily','lineHeight','elevation','shapeAppearance','shapeAppearanceOverlay','tooltipText'} | SHAPE_APPEARANCE_KEYS | APP_DRAWABLE_COMPAT_ATTRS | APP_LAYOUT_PARAMS_ATTRS | APP_AUTOSIZE_ATTRS | BADGE_EXT_ATTRS
TOOLBAR_APP_EXTRA_ATTRS={'popupTheme','menu','titleCentered','subtitleCentered','navigationIconTint'}
CONSTRAINT_ANCHORS={'layout_constraintStart_toStartOf':196,'layout_constraintLeft_toLeftOf':196,'layout_constraintStart_toEndOf':197,'layout_constraintLeft_toRightOf':197,'layout_constraintEnd_toEndOf':198,'layout_constraintRight_toRightOf':198,'layout_constraintEnd_toStartOf':199,'layout_constraintRight_toLeftOf':199,'layout_constraintTop_toTopOf':200,'layout_constraintTop_toBottomOf':201,'layout_constraintBottom_toBottomOf':202,'layout_constraintBottom_toTopOf':203}
CONSTRAINT_BIASES={'layout_constraintHorizontal_bias':204,'layout_constraintVertical_bias':205}
CONSTRAINT_CHAIN_STYLES={'layout_constraintHorizontal_chainStyle':214,'layout_constraintVertical_chainStyle':215}
CONSTRAINT_WEIGHTS={'layout_constraintHorizontal_weight':216,'layout_constraintVertical_weight':217}
THEME_COLORS={
    'colorPrimary':'#6750A4','colorOnPrimary':'#FFFFFF','colorPrimaryContainer':'#EADDFF','colorOnPrimaryContainer':'#21005D',
    'colorSecondary':'#625B71','colorOnSecondary':'#FFFFFF','colorSecondaryContainer':'#E8DEF8','colorOnSecondaryContainer':'#1D192B',
    'colorTertiary':'#7D5260','colorOnTertiary':'#FFFFFF','colorTertiaryContainer':'#FFD8E4','colorOnTertiaryContainer':'#31111D',
    'colorError':'#B3261E','colorOnError':'#FFFFFF','colorErrorContainer':'#F9DEDC','colorOnErrorContainer':'#410E0B',
    'colorBackground':'#FFFBFE','colorOnBackground':'#1C1B1F',
    'colorSurface':'#FEF7FF','colorOnSurface':'#1D1B20','colorSurfaceVariant':'#E7E0EC','colorOnSurfaceVariant':'#49454F',
    'colorSurfaceBright':'#FEF7FF','colorSurfaceDim':'#DED8E1','colorSurfaceTint':'#6750A4',
    'colorSurfaceContainerLowest':'#FFFFFF','colorSurfaceContainerLow':'#F7F2FA','colorSurfaceContainer':'#F3EDF7',
    'colorSurfaceContainerHigh':'#ECE6F0','colorSurfaceContainerHighest':'#E6E0E9',
    'colorOutline':'#79747E','colorOutlineVariant':'#CAC4D0',
    'colorInverseSurface':'#322F35','colorInverseOnSurface':'#F5EFF7','colorInversePrimary':'#D0BCFF',
    'colorPrimaryFixed':'#EADDFF','colorPrimaryFixedDim':'#D0BCFF','colorOnPrimaryFixed':'#21005D','colorOnPrimaryFixedVariant':'#4F378B',
    'colorSecondaryFixed':'#E8DEF8','colorSecondaryFixedDim':'#CCC2DC','colorOnSecondaryFixed':'#1D192B','colorOnSecondaryFixedVariant':'#4A4458',
    'colorTertiaryFixed':'#FFD8E4','colorTertiaryFixedDim':'#EFB8C8','colorOnTertiaryFixed':'#31111D','colorOnTertiaryFixedVariant':'#633B48',
    'colorAccent':'#6750A4','colorPrimaryDark':'#4F378B','colorPrimaryVariant':'#4F378B','colorSecondaryVariant':'#4A4458',
    'colorControlNormal':'#49454F','colorControlActivated':'#6750A4','colorControlHighlight':'#1F6750A4',
    'colorBackgroundFloating':'#FEF7FF','colorButtonNormal':'#E7E0EC','colorSwitchThumbNormal':'#FEF7FF',
    'statusBarColor':'#4F378B','navigationBarColor':'#000000',
    'textColorPrimary':'#1D1B20','textColorSecondary':'#49454F','textColorTertiary':'#79747E','textColorHint':'#79747E',
    'textColorPrimaryInverse':'#F5EFF7','textColorSecondaryInverse':'#CAC4D0','textColorTertiaryInverse':'#938F99','textColorHintInverse':'#CAC4D0',
    'textColorHighlight':'#666750A4','textColorLink':'#6750A4',
    'windowBackground':'#FEF7FF','selectableItemBackground':'#00000000','selectableItemBackgroundBorderless':'#00000000',
    'controlBackground':'#00000000','actionBarItemBackground':'#00000000',
    'listDivider':'#CAC4D0','dividerHorizontal':'#CAC4D0','dividerVertical':'#CAC4D0',
}
THEME_DIMENS={
    'actionBarSize':'56dp','listPreferredItemHeight':'64dp','listPreferredItemHeightSmall':'48dp',
    'listPreferredItemHeightLarge':'80dp','listPreferredItemPaddingStart':'16dp','listPreferredItemPaddingEnd':'16dp',
    'listPreferredItemPaddingLeft':'16dp','listPreferredItemPaddingRight':'16dp','dialogPreferredPadding':'24dp',
    'dropdownListPreferredItemHeight':'48dp','dialogCornerRadius':'28dp','buttonCornerRadius':'20dp',
    'listDivider':'1dp','actionBarDivider':'0dp','scrollbarSize':'4dp',
    'shapeCornerSizeExtraSmall':'4dp','shapeCornerSizeSmall':'8dp','shapeCornerSizeMedium':'12dp',
    'shapeCornerSizeLarge':'16dp','shapeCornerSizeLargeIncreased':'20dp','shapeCornerSizeExtraLarge':'28dp',
    'shapeCornerSizeExtraLargeIncreased':'32dp','shapeCornerSizeExtraExtraLarge':'48dp',
}
ANDROID_COLORS={
    '@android:color/transparent':'#00000000','@android:color/white':'#FFFFFFFF','@android:color/black':'#FF000000',
    '@android:color/darker_gray':'#FFAAAAAA','@android:color/background_dark':'#FF000000','@android:color/background_light':'#FFFFFFFF',
    '@android:color/primary_text_light':'#FF000000','@android:color/primary_text_dark':'#FFFFFFFF',
    '@android:color/secondary_text_light':'#8A000000','@android:color/secondary_text_dark':'#B3FFFFFF',
    '@android:color/tertiary_text_light':'#FF808080','@android:color/tertiary_text_dark':'#FF808080',
    '@android:color/primary_text_light_nodisable':'#FF000000','@android:color/primary_text_dark_nodisable':'#FFFFFFFF',
    '@android:color/secondary_text_light_nodisable':'#8A000000','@android:color/secondary_text_dark_nodisable':'#B3FFFFFF',
    '@android:color/holo_blue_light':'#FF33B5E5','@android:color/holo_blue_dark':'#FF0099CC',
    '@android:color/holo_green_light':'#FF99CC00','@android:color/holo_green_dark':'#FF669900',
    '@android:color/holo_red_light':'#FFFF4444','@android:color/holo_red_dark':'#FFCC0000',
    '@android:color/holo_orange_light':'#FFFFBB33','@android:color/holo_orange_dark':'#FFFF8800',
    '@android:color/holo_purple':'#FFAA66CC','@android:color/tab_indicator_text':'#FF808080',
}
ANDROID_STRINGS={
    '@android:string/ok':'OK','@android:string/cancel':'Cancel','@android:string/yes':'Yes','@android:string/no':'No',
    '@android:string/copy':'Copy','@android:string/paste':'Paste','@android:string/cut':'Cut',
    '@android:string/selectAll':'Select all','@android:string/untitled':'<Untitled>','@android:string/unknownName':'(Unknown)',
    '@android:string/search_go':'Search',
}
ANDROID_DIMENS={
    '@android:dimen/app_icon_size':'48dp','@android:dimen/notification_large_icon_width':'64dp',
    '@android:dimen/notification_large_icon_height':'64dp','@android:dimen/thumbnail_width':'192dp',
    '@android:dimen/thumbnail_height':'192dp',
}
ANDROID_INTEGERS={
    '@android:integer/config_shortAnimTime':'200','@android:integer/config_mediumAnimTime':'400',
    '@android:integer/config_longAnimTime':'500',
}
# Pinned from context/reference-sources/android-framework/res/values/dimens_material.xml
# and context/reference-repos/material-android/lib/java/com/google/android/material/*/res/values/{dimens,tokens*}.xml
BUILTIN_LIBRARY_DIMENS={
    '@dimen/abc_action_bar_content_inset_material':'16dp','@dimen/abc_action_bar_content_inset_with_nav':'72dp','@dimen/abc_action_bar_default_height_material':'56dp','@dimen/abc_action_bar_default_padding_end_material':'0dp',
    '@dimen/abc_action_bar_default_padding_start_material':'0dp','@dimen/abc_action_bar_elevation_material':'4dp','@dimen/abc_action_bar_icon_vertical_padding_material':'16dp','@dimen/abc_action_bar_overflow_padding_end_material':'10dp',
    '@dimen/abc_action_bar_overflow_padding_start_material':'6dp','@dimen/abc_action_bar_subtitle_bottom_margin_material':'5dp','@dimen/abc_action_bar_subtitle_top_margin_material':'-3dp','@dimen/abc_action_button_min_height_material':'48dp',
    '@dimen/abc_action_button_min_width_material':'48dp','@dimen/abc_action_button_min_width_overflow_material':'36dp','@dimen/abc_button_elevation_material':'2dp','@dimen/abc_button_inset_horizontal_material':'@dimen/abc_control_inset_material',
    '@dimen/abc_button_inset_vertical_material':'6dp','@dimen/abc_button_padding_horizontal_material':'8dp','@dimen/abc_button_padding_vertical_material':'@dimen/abc_control_padding_material','@dimen/abc_button_pressed_z_material':'4dp',
    '@dimen/abc_control_corner_material':'2dp','@dimen/abc_control_inset_material':'4dp','@dimen/abc_control_padding_material':'4dp','@dimen/abc_dialog_list_padding_bottom_no_buttons':'8dp',
    '@dimen/abc_dialog_list_padding_top_no_title':'8dp','@dimen/abc_dialog_padding_material':'24dp','@dimen/abc_dialog_padding_top_material':'18dp','@dimen/abc_dialog_title_divider_material':'8dp',
    '@dimen/abc_edit_text_inset_bottom_material':'7dp','@dimen/abc_edit_text_inset_horizontal_material':'4dp','@dimen/abc_edit_text_inset_top_material':'10dp','@dimen/abc_floating_window_margin_bottom':'32dp',
    '@dimen/abc_floating_window_margin_left':'16dp','@dimen/abc_floating_window_margin_right':'16dp','@dimen/abc_floating_window_margin_top':'8dp','@dimen/abc_floating_window_z':'16dp',
    '@dimen/abc_list_item_padding_end_material':'@dimen/abc_action_bar_content_inset_material','@dimen/abc_list_item_padding_horizontal_material':'@dimen/abc_action_bar_content_inset_material','@dimen/abc_list_item_padding_start_material':'@dimen/abc_action_bar_content_inset_material','@dimen/abc_progress_bar_corner_material':'0dp',
    '@dimen/abc_progress_bar_height_material':'4dp','@dimen/abc_progress_bar_size_large':'76dp','@dimen/abc_progress_bar_size_medium':'48dp','@dimen/abc_progress_bar_size_small':'16dip',
    '@dimen/abc_seekbar_track_background_height_material':'2dp','@dimen/abc_seekbar_track_progress_height_material':'2dp','@dimen/abc_select_dialog_drawable_padding_start_material':'20dp','@dimen/abc_select_dialog_padding_start_material':'20dp',
    '@dimen/abc_text_size_body_1_material':'14sp','@dimen/abc_text_size_body_2_material':'14sp','@dimen/abc_text_size_button_material':'14sp','@dimen/abc_text_size_caption_material':'12sp',
    '@dimen/abc_text_size_display_1_material':'34sp','@dimen/abc_text_size_display_2_material':'45sp','@dimen/abc_text_size_display_3_material':'56sp','@dimen/abc_text_size_display_4_material':'112sp',
    '@dimen/abc_text_size_headline_material':'24sp','@dimen/abc_text_size_large_material':'22sp','@dimen/abc_text_size_medium_material':'18sp','@dimen/abc_text_size_menu_header_material':'14sp',
    '@dimen/abc_text_size_menu_material':'16sp','@dimen/abc_text_size_small_material':'14sp','@dimen/abc_text_size_subhead_material':'16sp','@dimen/abc_text_size_subtitle_material_toolbar':'16dp',
    '@dimen/abc_text_size_title_material':'20sp','@dimen/abc_text_size_title_material_toolbar':'20dp','@dimen/abc_alert_dialog_button_bar_height':'48dp','@dimen/abc_alert_dialog_button_dimen':'48dp',
    '@dimen/abc_config_prefDialogWidth':'320dp','@dimen/abc_dialog_corner_radius_material':'2dp','@dimen/abc_dialog_fixed_width_major':'320dp','@dimen/abc_dialog_fixed_width_minor':'320dp',
    '@dimen/abc_dropdownitem_icon_width':'32dp','@dimen/abc_dropdownitem_text_padding_left':'8dp','@dimen/abc_dropdownitem_text_padding_right':'8dp','@dimen/abc_search_view_preferred_height':'48dp',
    '@dimen/abc_search_view_preferred_width':'320dp','@dimen/abc_star_big':'48dp','@dimen/abc_star_medium':'36dp','@dimen/abc_star_small':'16dp',
    '@dimen/abc_switch_padding':'0px','@dimen/abc_action_bar_stacked_max_height':'48dp','@dimen/abc_action_bar_stacked_tab_max_width':'180dp','@dimen/abc_disabled_alpha_material_light':'0.26',
    '@dimen/abc_disabled_alpha_material_dark':'0.30','@dimen/design_appbar_elevation':'4dp','@dimen/design_bottom_navigation_active_item_max_width':'168dp','@dimen/design_bottom_navigation_active_item_min_width':'96dp',
    '@dimen/design_bottom_navigation_active_text_size':'14sp','@dimen/design_bottom_navigation_elevation':'8dp','@dimen/design_bottom_navigation_height':'56dp','@dimen/design_bottom_navigation_icon_size':'24dp',
    '@dimen/design_bottom_navigation_item_max_width':'96dp','@dimen/design_bottom_navigation_item_min_width':'56dp','@dimen/design_bottom_navigation_label_padding':'10dp','@dimen/design_bottom_navigation_margin':'8dp',
    '@dimen/design_bottom_navigation_shadow_height':'1dp','@dimen/design_bottom_navigation_text_size':'12sp','@dimen/design_bottom_sheet_elevation':'8dp','@dimen/design_bottom_sheet_modal_elevation':'16dp',
    '@dimen/design_bottom_sheet_peek_height_min':'64dp','@dimen/design_fab_border_width':'0.5dp','@dimen/design_fab_elevation':'6dp','@dimen/design_fab_image_size':'24dp',
    '@dimen/design_fab_size_mini':'40dp','@dimen/design_fab_size_normal':'56dp','@dimen/design_fab_translation_z_hovered_focused':'6dp','@dimen/design_fab_translation_z_pressed':'6dp',
    '@dimen/design_navigation_elevation':'16dp','@dimen/design_navigation_icon_padding':'32dp','@dimen/design_navigation_icon_size':'24dp','@dimen/design_navigation_item_horizontal_padding':'16dp',
    '@dimen/design_navigation_item_icon_padding':'32dp','@dimen/design_navigation_item_vertical_padding':'4dp','@dimen/design_navigation_max_width':'280dp','@dimen/design_navigation_padding_bottom':'8dp',
    '@dimen/design_navigation_separator_vertical_padding':'8dp','@dimen/design_snackbar_action_inline_max_width':'128dp','@dimen/design_snackbar_action_text_color_alpha':'1.0','@dimen/design_snackbar_background_corner_radius':'0dp',
    '@dimen/design_snackbar_elevation':'6dp','@dimen/design_snackbar_extra_spacing_horizontal':'0dp','@dimen/design_snackbar_max_width':'-1px','@dimen/design_snackbar_min_width':'-1px',
    '@dimen/design_snackbar_padding_horizontal':'12dp','@dimen/design_snackbar_padding_vertical':'14dp','@dimen/design_snackbar_padding_vertical_2lines':'16dp','@dimen/design_snackbar_text_size':'14sp',
    '@dimen/design_tab_max_width':'264dp','@dimen/design_tab_scrollable_min_width':'72dp','@dimen/design_tab_text_size':'14sp','@dimen/design_tab_text_size_2line':'12sp',
    '@dimen/design_textinput_caption_translate_y':'5dp','@dimen/m3_appbar_expanded_title_margin_bottom':'16dp','@dimen/m3_appbar_expanded_title_margin_horizontal':'16dp','@dimen/m3_appbar_scrim_height_trigger':'96dp',
    '@dimen/m3_appbar_scrim_height_trigger_large':'112dp','@dimen/m3_appbar_scrim_height_trigger_medium':'112dp','@dimen/m3_appbar_size_compact':'64dp','@dimen/m3_appbar_size_large':'152dp',
    '@dimen/m3_appbar_size_large_with_subtitle':'152dp','@dimen/m3_appbar_size_medium':'112dp','@dimen/m3_appbar_size_medium_with_subtitle':'136dp','@dimen/m3_badge_horizontal_offset':'1.5dp',
    '@dimen/m3_badge_offset':'6dp','@dimen/m3_badge_size':'6dp','@dimen/m3_badge_vertical_offset':'1.5dp','@dimen/m3_badge_with_text_horizontal_offset':'3dp',
    '@dimen/m3_badge_with_text_offset':'12dp','@dimen/m3_badge_with_text_size':'16dp','@dimen/m3_badge_with_text_vertical_offset':'4dp','@dimen/m3_badge_with_text_vertical_padding':'2dp',
    '@dimen/m3_bottom_nav_item_active_indicator_height':'32dp','@dimen/m3_bottom_nav_item_active_indicator_margin_horizontal':'4dp','@dimen/m3_bottom_nav_item_active_indicator_width':'64dp','@dimen/m3_bottom_nav_item_expanded_active_indicator_height':'40dp',
    '@dimen/m3_bottom_nav_item_padding_bottom':'16dp','@dimen/m3_bottom_nav_item_padding_top':'12dp','@dimen/m3_bottom_nav_min_height':'80dp','@dimen/m3_bottom_sheet_drag_handle_bottom_padding':'20dp',
    '@dimen/m3_bottom_sheet_elevation':'1dp','@dimen/m3_bottom_sheet_modal_elevation':'1dp','@dimen/m3_btn_dialog_btn_min_width':'64dp','@dimen/m3_btn_dialog_btn_spacing':'8dp',
    '@dimen/m3_btn_disabled_elevation':'0dp','@dimen/m3_btn_disabled_translation_z':'0dp','@dimen/m3_btn_elevated_translation_z_hovered':'2dp','@dimen/m3_btn_elevation':'0dp',
    '@dimen/m3_btn_group_overflow_item_icon_horizontal_padding':'4dp','@dimen/m3_btn_icon_btn_padding_left':'16dp','@dimen/m3_btn_icon_btn_padding_right':'24dp','@dimen/m3_btn_icon_only_default_padding':'10dp',
    '@dimen/m3_btn_icon_only_default_size':'20dp','@dimen/m3_btn_icon_only_icon_padding':'0dp','@dimen/m3_btn_icon_only_min_width':'20dp','@dimen/m3_btn_inset':'4dp',
    '@dimen/m3_btn_max_width':'320dp','@dimen/m3_btn_padding_bottom':'6dp','@dimen/m3_btn_padding_left':'24dp','@dimen/m3_btn_padding_right':'24dp',
    '@dimen/m3_btn_padding_top':'6dp','@dimen/m3_btn_stroke_size':'1dp','@dimen/m3_btn_text_btn_icon_padding_left':'12dp','@dimen/m3_btn_text_btn_icon_padding_right':'16dp',
    '@dimen/m3_btn_text_btn_padding_left':'12dp','@dimen/m3_btn_text_btn_padding_right':'12dp','@dimen/m3_btn_translation_z_base':'0dp','@dimen/m3_btn_translation_z_hovered':'1dp',
    '@dimen/m3_card_disabled_z':'0dp','@dimen/m3_card_dragged_z':'6dp','@dimen/m3_card_elevated_disabled_z':'0dp','@dimen/m3_card_elevated_dragged_z':'7dp',
    '@dimen/m3_card_elevated_elevation':'1dp','@dimen/m3_card_elevated_hovered_z':'2dp','@dimen/m3_card_elevation':'0dp','@dimen/m3_card_hovered_z':'1dp',
    '@dimen/m3_card_stroke_width':'1dp','@dimen/m3_chip_checked_hovered_translation_z':'1dp','@dimen/m3_chip_corner_size':'8dp','@dimen/m3_chip_disabled_translation_z':'-1dp',
    '@dimen/m3_chip_dragged_translation_z':'7dp','@dimen/m3_chip_elevated_elevation':'1dp','@dimen/m3_chip_hovered_translation_z':'2dp','@dimen/m3_chip_icon_size':'18dp',
    '@dimen/m3_divider_heavy_thickness':'8dp','@dimen/m3_fab_border_width':'0dp','@dimen/m3_fab_disabled_elevation':'0dp','@dimen/m3_fab_disabled_translation_z':'0dp',
    '@dimen/m3_fab_translation_z_base':'0dp','@dimen/m3_fab_translation_z_focused':'0dp','@dimen/m3_fab_translation_z_hovered':'2dp','@dimen/m3_fab_translation_z_pressed':'0dp',
    '@dimen/m3_menu_elevation':'3dp','@dimen/m3_navigation_content_horizontal_margin':'20dp','@dimen/m3_navigation_divider_bottom_margin':'3dp','@dimen/m3_navigation_divider_top_margin':'8dp',
    '@dimen/m3_navigation_drawer_layout_corner_size':'16dp','@dimen/m3_navigation_item_active_indicator_label_padding':'4dp','@dimen/m3_navigation_item_expanded_active_indicator_height_default':'40dp','@dimen/m3_navigation_item_horizontal_padding':'28dp',
    '@dimen/m3_navigation_item_icon_padding':'12dp','@dimen/m3_navigation_item_leading_trailing_space':'16dp','@dimen/m3_navigation_item_shape_inset_bottom':'0dp','@dimen/m3_navigation_item_shape_inset_end':'12dp',
    '@dimen/m3_navigation_item_shape_inset_start':'12dp','@dimen/m3_navigation_item_shape_inset_top':'0dp','@dimen/m3_navigation_item_vertical_padding':'4dp','@dimen/m3_navigation_menu_divider_horizontal_padding':'28dp',
    '@dimen/m3_navigation_menu_headline_horizontal_padding':'28dp','@dimen/m3_navigation_rail_default_width':'80dp','@dimen/m3_navigation_rail_elevation':'0dp','@dimen/m3_navigation_rail_expanded_active_indicator_height':'56dp',
    '@dimen/m3_navigation_rail_expanded_item_spacing':'0dp','@dimen/m3_navigation_rail_expanded_leading_trailing_space':'20dp','@dimen/m3_navigation_rail_icon_label_horizontal_padding':'8dp','@dimen/m3_navigation_rail_icon_label_padding':'4dp',
    '@dimen/m3_navigation_rail_icon_size':'24dp','@dimen/m3_navigation_rail_item_active_indicator_height':'32dp','@dimen/m3_navigation_rail_item_active_indicator_margin_horizontal':'4dp','@dimen/m3_navigation_rail_item_active_indicator_width':'56dp',
    '@dimen/m3_navigation_rail_item_min_height':'60dp','@dimen/m3_navigation_rail_item_padding_bottom':'12dp','@dimen/m3_navigation_rail_item_padding_bottom_with_large_font':'20dp','@dimen/m3_navigation_rail_item_padding_top':'4dp',
    '@dimen/m3_navigation_rail_item_padding_top_with_large_font':'12dp','@dimen/m3_navigation_rail_label_padding_horizontal':'2dp','@dimen/m3_navigation_rail_max_expanded_width':'360dp','@dimen/m3_navigation_rail_min_expanded_width':'220dp',
    '@dimen/m3_navigation_subheader_horizontal_padding':'16dp','@dimen/m3_navigation_subheader_top_margin':'12dp','@dimen/m3_navigation_subheader_vertical_padding':'8dp','@dimen/m3_searchbar_elevation':'6dp',
    '@dimen/m3_searchbar_height':'56dp','@dimen/m3_searchbar_margin_horizontal':'16dp','@dimen/m3_searchbar_margin_vertical':'16dp','@dimen/m3_searchbar_outlined_stroke_width':'1dp',
    '@dimen/m3_searchbar_padding_start':'4dp','@dimen/m3_searchbar_parent_width_breakpoint':'312dp','@dimen/m3_searchbar_text_margin_start_no_navigation_icon':'16dp','@dimen/m3_searchbar_text_size':'16sp',
    '@dimen/m3_searchview_divider_size':'1dp','@dimen/m3_searchview_elevation':'6dp','@dimen/m3_searchview_height':'72dp','@dimen/m3_side_sheet_margin_detached':'16dp',
    '@dimen/m3_side_sheet_modal_elevation':'1dp','@dimen/m3_side_sheet_standard_elevation':'0dp','@dimen/m3_side_sheet_width':'256dp','@dimen/m3_slider_focus_ring_padding':'2dp',
    '@dimen/m3_slider_focus_ring_thumb_height_decrease':'6dp','@dimen/m3_slider_thumb_elevation':'2dp','@dimen/m3_slider_track_icon_padding':'10dp','@dimen/m3_snackbar_action_text_color_alpha':'1.0',
    '@dimen/m3_snackbar_margin':'8dp','@dimen/m3_sys_elevation_level0':'0dp','@dimen/m3_sys_elevation_level1':'1dp','@dimen/m3_sys_elevation_level2':'3dp',
    '@dimen/m3_sys_elevation_level3':'6dp','@dimen/m3_sys_elevation_level4':'8dp','@dimen/m3_sys_elevation_level5':'12dp','@dimen/m3_sys_motion_easing_emphasized_accelerate_control_x1':'0.3',
    '@dimen/m3_sys_motion_easing_emphasized_accelerate_control_x2':'0.8','@dimen/m3_sys_motion_easing_emphasized_accelerate_control_y1':'0','@dimen/m3_sys_motion_easing_emphasized_accelerate_control_y2':'0.2','@dimen/m3_sys_motion_easing_emphasized_decelerate_control_x1':'0.1',
    '@dimen/m3_sys_motion_easing_emphasized_decelerate_control_x2':'0.1','@dimen/m3_sys_motion_easing_emphasized_decelerate_control_y1':'0.7','@dimen/m3_sys_motion_easing_emphasized_decelerate_control_y2':'1','@dimen/m3_sys_motion_easing_legacy_accelerate_control_x1':'0.4',
    '@dimen/m3_sys_motion_easing_legacy_accelerate_control_x2':'1','@dimen/m3_sys_motion_easing_legacy_accelerate_control_y1':'0','@dimen/m3_sys_motion_easing_legacy_accelerate_control_y2':'1','@dimen/m3_sys_motion_easing_legacy_control_x1':'0.4',
    '@dimen/m3_sys_motion_easing_legacy_control_x2':'0.2','@dimen/m3_sys_motion_easing_legacy_control_y1':'0','@dimen/m3_sys_motion_easing_legacy_control_y2':'1','@dimen/m3_sys_motion_easing_legacy_decelerate_control_x1':'0',
    '@dimen/m3_sys_motion_easing_legacy_decelerate_control_x2':'0.2','@dimen/m3_sys_motion_easing_legacy_decelerate_control_y1':'0','@dimen/m3_sys_motion_easing_legacy_decelerate_control_y2':'1','@dimen/m3_sys_motion_easing_linear_control_x1':'0',
    '@dimen/m3_sys_motion_easing_linear_control_x2':'1','@dimen/m3_sys_motion_easing_linear_control_y1':'0','@dimen/m3_sys_motion_easing_linear_control_y2':'1','@dimen/m3_sys_motion_easing_standard_accelerate_control_x1':'0.3',
    '@dimen/m3_sys_motion_easing_standard_accelerate_control_x2':'1','@dimen/m3_sys_motion_easing_standard_accelerate_control_y1':'0','@dimen/m3_sys_motion_easing_standard_accelerate_control_y2':'1','@dimen/m3_sys_motion_easing_standard_control_x1':'0.2',
    '@dimen/m3_sys_motion_easing_standard_control_x2':'0','@dimen/m3_sys_motion_easing_standard_control_y1':'0','@dimen/m3_sys_motion_easing_standard_control_y2':'1','@dimen/m3_sys_motion_easing_standard_decelerate_control_x1':'0',
    '@dimen/m3_sys_motion_easing_standard_decelerate_control_x2':'0','@dimen/m3_sys_motion_easing_standard_decelerate_control_y1':'0','@dimen/m3_sys_motion_easing_standard_decelerate_control_y2':'1','@dimen/m3_sys_motion_expressive_spring_default_effects_damping':'1',
    '@dimen/m3_sys_motion_expressive_spring_default_effects_stiffness':'1600','@dimen/m3_sys_motion_expressive_spring_default_spatial_damping':'0.8','@dimen/m3_sys_motion_expressive_spring_default_spatial_stiffness':'380','@dimen/m3_sys_motion_expressive_spring_fast_effects_damping':'1',
    '@dimen/m3_sys_motion_expressive_spring_fast_effects_stiffness':'3800','@dimen/m3_sys_motion_expressive_spring_fast_spatial_damping':'0.6','@dimen/m3_sys_motion_expressive_spring_fast_spatial_stiffness':'800','@dimen/m3_sys_motion_expressive_spring_slow_effects_damping':'1',
    '@dimen/m3_sys_motion_expressive_spring_slow_effects_stiffness':'800','@dimen/m3_sys_motion_expressive_spring_slow_spatial_damping':'0.8','@dimen/m3_sys_motion_expressive_spring_slow_spatial_stiffness':'200','@dimen/m3_sys_motion_standard_spring_default_effects_damping':'1',
    '@dimen/m3_sys_motion_standard_spring_default_effects_stiffness':'1600','@dimen/m3_sys_motion_standard_spring_default_spatial_damping':'0.9','@dimen/m3_sys_motion_standard_spring_default_spatial_stiffness':'700','@dimen/m3_sys_motion_standard_spring_fast_effects_damping':'1',
    '@dimen/m3_sys_motion_standard_spring_fast_effects_stiffness':'3800','@dimen/m3_sys_motion_standard_spring_fast_spatial_damping':'0.9','@dimen/m3_sys_motion_standard_spring_fast_spatial_stiffness':'1400','@dimen/m3_sys_motion_standard_spring_slow_effects_damping':'1',
    '@dimen/m3_sys_motion_standard_spring_slow_effects_stiffness':'800','@dimen/m3_sys_motion_standard_spring_slow_spatial_damping':'0.9','@dimen/m3_sys_motion_standard_spring_slow_spatial_stiffness':'300','@dimen/m3_sys_shape_corner_value_extra_extra_large':'48dp',
    '@dimen/m3_sys_shape_corner_value_extra_large':'28dp','@dimen/m3_sys_shape_corner_value_extra_large_increased':'32dp','@dimen/m3_sys_shape_corner_value_extra_small':'4dp','@dimen/m3_sys_shape_corner_value_large':'16dp',
    '@dimen/m3_sys_shape_corner_value_large_increased':'20dp','@dimen/m3_sys_shape_corner_value_medium':'12dp','@dimen/m3_sys_shape_corner_value_none':'0dp','@dimen/m3_sys_shape_corner_value_small':'8dp',
    '@dimen/m3_sys_state_disabled_state_layer_opacity':'0.38','@dimen/m3_sys_state_dragged_state_layer_opacity':'0.16','@dimen/m3_sys_state_focus_state_layer_opacity':'0.1','@dimen/m3_sys_state_hover_state_layer_opacity':'0.08',
    '@dimen/m3_sys_state_pressed_state_layer_opacity':'0.1','@dimen/m3_toolbar_button_circle_size':'40dp','@dimen/m3_toolbar_text_size_title':'20dp','@dimen/material_bottom_sheet_max_width':'640dp',
    '@dimen/material_clock_display_height':'80dp','@dimen/material_clock_display_padding':'24dp','@dimen/material_clock_display_width':'96dp','@dimen/material_clock_face_margin_bottom':'24dp',
    '@dimen/material_clock_face_margin_top':'44dp','@dimen/material_clock_hand_center_dot_radius':'4dp','@dimen/material_clock_hand_padding':'4dp','@dimen/material_clock_hand_stroke_width':'2dp',
    '@dimen/material_clock_number_text_size':'15sp','@dimen/material_clock_period_toggle_height':'98dp','@dimen/material_clock_period_toggle_horizontal_gap':'12dp','@dimen/material_clock_period_toggle_vertical_gap':'12dp',
    '@dimen/material_clock_period_toggle_width':'52dp','@dimen/material_clock_size':'256dp','@dimen/material_cursor_inset':'-4dp','@dimen/material_cursor_width':'2dp',
    '@dimen/material_divider_thickness':'1dp','@dimen/material_emphasis_disabled':'0.38','@dimen/material_emphasis_disabled_background':'0.12','@dimen/material_emphasis_high_type':'0.87',
    '@dimen/material_emphasis_medium':'0.6','@dimen/material_filled_edittext_font_1_3_padding_bottom':'12dp','@dimen/material_filled_edittext_font_1_3_padding_top':'23dp','@dimen/material_filled_edittext_font_2_0_padding_bottom':'8dp',
    '@dimen/material_filled_edittext_font_2_0_padding_top':'32dp','@dimen/material_font_1_3_box_collapsed_padding_top':'4dp','@dimen/material_font_2_0_box_collapsed_padding_top':'8dp','@dimen/material_helper_text_default_padding_top':'4dp',
    '@dimen/material_helper_text_font_1_3_padding_horizontal':'12dp','@dimen/material_helper_text_font_1_3_padding_top':'8dp','@dimen/material_input_text_to_prefix_suffix_padding':'2dp','@dimen/material_textinput_default_width':'245dp',
    '@dimen/material_textinput_max_width':'488dp','@dimen/material_textinput_min_width':'56dp','@dimen/material_textinput_min_width_with_label':'88dp','@dimen/material_time_picker_minimum_screen_height':'560dp',
    '@dimen/material_time_picker_minimum_screen_width':'340dp','@dimen/mtrl_alert_dialog_background_inset_bottom':'10dp','@dimen/mtrl_alert_dialog_background_inset_end':'10dp','@dimen/mtrl_alert_dialog_background_inset_start':'10dp',
    '@dimen/mtrl_alert_dialog_background_inset_top':'10dp','@dimen/mtrl_alert_dialog_picker_background_inset':'24dp','@dimen/mtrl_badge_horizontal_edge_offset':'4dp','@dimen/mtrl_badge_long_text_horizontal_padding':'4dp',
    '@dimen/mtrl_badge_size':'8dp','@dimen/mtrl_badge_text_horizontal_edge_offset':'6dp','@dimen/mtrl_badge_text_size':'10sp','@dimen/mtrl_badge_toolbar_action_menu_item_horizontal_offset':'12dp',
    '@dimen/mtrl_badge_toolbar_action_menu_item_vertical_offset':'12dp','@dimen/mtrl_badge_with_text_size':'16dp','@dimen/mtrl_bottomappbar_fabOffsetEndMode':'60dp','@dimen/mtrl_bottomappbar_fab_bottom_margin':'16dp',
    '@dimen/mtrl_bottomappbar_fab_cradle_margin':'5dp','@dimen/mtrl_bottomappbar_fab_cradle_rounded_corner_radius':'8dp','@dimen/mtrl_bottomappbar_fab_cradle_vertical_offset':'0dp','@dimen/mtrl_bottomappbar_height':'56dp',
    '@dimen/mtrl_btn_corner_radius':'4dp','@dimen/mtrl_btn_dialog_btn_min_width':'64dp','@dimen/mtrl_btn_disabled_elevation':'0dp','@dimen/mtrl_btn_disabled_z':'0dp',
    '@dimen/mtrl_btn_elevation':'2dp','@dimen/mtrl_btn_focused_z':'2dp','@dimen/mtrl_btn_hovered_z':'2dp','@dimen/mtrl_btn_icon_btn_padding_left':'12dp',
    '@dimen/mtrl_btn_icon_padding':'8dp','@dimen/mtrl_btn_inset':'6dp','@dimen/mtrl_btn_letter_spacing':'0.07','@dimen/mtrl_btn_max_width':'320dp',
    '@dimen/mtrl_btn_padding_bottom':'4dp','@dimen/mtrl_btn_padding_left':'16dp','@dimen/mtrl_btn_padding_right':'16dp','@dimen/mtrl_btn_padding_top':'4dp',
    '@dimen/mtrl_btn_pressed_z':'6dp','@dimen/mtrl_btn_snackbar_margin_horizontal':'8dp','@dimen/mtrl_btn_stroke_size':'1dp','@dimen/mtrl_btn_text_btn_icon_padding':'4dp',
    '@dimen/mtrl_btn_text_btn_padding_left':'8dp','@dimen/mtrl_btn_text_btn_padding_right':'8dp','@dimen/mtrl_btn_text_size':'14sp','@dimen/mtrl_btn_z':'0dp',
    '@dimen/mtrl_calendar_action_confirm_button_min_width':'64dp','@dimen/mtrl_calendar_action_height':'52dp','@dimen/mtrl_calendar_action_padding':'8dp','@dimen/mtrl_calendar_bottom_padding':'0dp',
    '@dimen/mtrl_calendar_content_padding':'12dp','@dimen/mtrl_calendar_day_corner':'15dp','@dimen/mtrl_calendar_day_height':'32dp','@dimen/mtrl_calendar_day_horizontal_padding':'3dp',
    '@dimen/mtrl_calendar_day_today_stroke':'1dp','@dimen/mtrl_calendar_day_vertical_padding':'1dp','@dimen/mtrl_calendar_day_width':'36dp','@dimen/mtrl_calendar_days_of_week_height':'24dp',
    '@dimen/mtrl_calendar_dialog_background_inset':'16dp','@dimen/mtrl_calendar_header_content_padding':'12dp','@dimen/mtrl_calendar_header_content_padding_fullscreen':'4dp','@dimen/mtrl_calendar_header_divider_thickness':'1dp',
    '@dimen/mtrl_calendar_header_height':'120dp','@dimen/mtrl_calendar_header_height_fullscreen':'128dp','@dimen/mtrl_calendar_header_selection_line_height':'32dp','@dimen/mtrl_calendar_header_text_padding':'12dp',
    '@dimen/mtrl_calendar_header_toggle_margin_bottom':'8dp','@dimen/mtrl_calendar_header_toggle_margin_top':'24dp','@dimen/mtrl_calendar_landscape_header_width':'0dp','@dimen/mtrl_calendar_maximum_default_fullscreen_minor_axis':'480dp',
    '@dimen/mtrl_calendar_month_horizontal_padding':'2dp','@dimen/mtrl_calendar_month_vertical_padding':'0dp','@dimen/mtrl_calendar_navigation_bottom_padding':'4dp','@dimen/mtrl_calendar_navigation_height':'48dp',
    '@dimen/mtrl_calendar_navigation_top_padding':'4dp','@dimen/mtrl_calendar_selection_baseline_to_top_fullscreen':'104dp','@dimen/mtrl_calendar_selection_text_baseline_to_bottom':'20dp','@dimen/mtrl_calendar_selection_text_baseline_to_bottom_fullscreen':'24dp',
    '@dimen/mtrl_calendar_selection_text_baseline_to_top':'100dp','@dimen/mtrl_calendar_text_input_padding_top':'16dp','@dimen/mtrl_calendar_title_baseline_to_top':'28dp','@dimen/mtrl_calendar_title_baseline_to_top_fullscreen':'68dp',
    '@dimen/mtrl_calendar_year_corner':'18dp','@dimen/mtrl_calendar_year_height':'52dp','@dimen/mtrl_calendar_year_horizontal_padding':'8dp','@dimen/mtrl_calendar_year_vertical_padding':'8dp',
    '@dimen/mtrl_calendar_year_width':'88dp','@dimen/mtrl_card_checked_icon_margin':'8dp','@dimen/mtrl_card_checked_icon_size':'24dp','@dimen/mtrl_card_corner_radius':'4dp',
    '@dimen/mtrl_card_dragged_z':'5dp','@dimen/mtrl_card_elevation':'1dp','@dimen/mtrl_card_spacing':'8dp','@dimen/mtrl_chip_pressed_translation_z':'3dp',
    '@dimen/mtrl_chip_text_size':'14sp','@dimen/mtrl_exposed_dropdown_menu_popup_elevation':'8dp','@dimen/mtrl_exposed_dropdown_menu_popup_vertical_offset':'1dp','@dimen/mtrl_exposed_dropdown_menu_popup_vertical_padding':'8dp',
    '@dimen/mtrl_extended_fab_bottom_padding':'12dp','@dimen/mtrl_extended_fab_disabled_elevation':'0dp','@dimen/mtrl_extended_fab_disabled_translation_z':'0dp','@dimen/mtrl_extended_fab_elevation':'6dp',
    '@dimen/mtrl_extended_fab_end_padding':'20dp','@dimen/mtrl_extended_fab_end_padding_icon':'20dp','@dimen/mtrl_extended_fab_icon_size':'24dp','@dimen/mtrl_extended_fab_icon_text_spacing':'12dp',
    '@dimen/mtrl_extended_fab_min_height':'48dp','@dimen/mtrl_extended_fab_min_width':'120dp','@dimen/mtrl_extended_fab_start_padding':'20dp','@dimen/mtrl_extended_fab_start_padding_icon':'12dp',
    '@dimen/mtrl_extended_fab_top_padding':'12dp','@dimen/mtrl_extended_fab_translation_z_base':'0dp','@dimen/mtrl_extended_fab_translation_z_hovered_focused':'2dp','@dimen/mtrl_extended_fab_translation_z_pressed':'6dp',
    '@dimen/mtrl_fab_elevation':'6dp','@dimen/mtrl_fab_min_touch_target':'48dp','@dimen/mtrl_fab_translation_z_hovered_focused':'2dp','@dimen/mtrl_fab_translation_z_pressed':'6dp',
    '@dimen/mtrl_focus_ring_inner_stroke_inset':'1dp','@dimen/mtrl_focus_ring_inner_stroke_width':'3dp','@dimen/mtrl_focus_ring_outer_stroke_width':'2dp','@dimen/mtrl_high_ripple_default_alpha':'0.48',
    '@dimen/mtrl_high_ripple_focused_alpha':'0.48','@dimen/mtrl_high_ripple_hovered_alpha':'0.16','@dimen/mtrl_high_ripple_pressed_alpha':'0.48','@dimen/mtrl_low_ripple_default_alpha':'0.24',
    '@dimen/mtrl_low_ripple_focused_alpha':'0.24','@dimen/mtrl_low_ripple_hovered_alpha':'0.08','@dimen/mtrl_low_ripple_pressed_alpha':'0.24','@dimen/mtrl_min_touch_target_size':'48dp',
    '@dimen/mtrl_navigation_bar_item_default_icon_size':'24dp','@dimen/mtrl_navigation_bar_item_default_margin':'8dp','@dimen/mtrl_navigation_elevation':'0dp','@dimen/mtrl_navigation_item_horizontal_padding':'22dp',
    '@dimen/mtrl_navigation_item_icon_padding':'14dp','@dimen/mtrl_navigation_item_icon_size':'24dp','@dimen/mtrl_navigation_item_shape_horizontal_margin':'8dp','@dimen/mtrl_navigation_item_shape_vertical_margin':'4dp',
    '@dimen/mtrl_navigation_rail_active_text_size':'14dp','@dimen/mtrl_navigation_rail_compact_width':'56dp','@dimen/mtrl_navigation_rail_default_width':'72dp','@dimen/mtrl_navigation_rail_elevation':'8dp',
    '@dimen/mtrl_navigation_rail_icon_margin':'14dp','@dimen/mtrl_navigation_rail_icon_size':'24dp','@dimen/mtrl_navigation_rail_margin':'8dp','@dimen/mtrl_navigation_rail_text_bottom_margin':'16dp',
    '@dimen/mtrl_navigation_rail_text_size':'12dp','@dimen/mtrl_progress_circular_inset':'4dp','@dimen/mtrl_progress_circular_inset_extra_small':'2dp','@dimen/mtrl_progress_circular_inset_medium':'4dp',
    '@dimen/mtrl_progress_circular_inset_small':'4dp','@dimen/mtrl_progress_circular_radius':'18dp','@dimen/mtrl_progress_circular_size':'40dp','@dimen/mtrl_progress_circular_size_extra_small':'20dp',
    '@dimen/mtrl_progress_circular_size_medium':'40dp','@dimen/mtrl_progress_circular_size_small':'28dp','@dimen/mtrl_progress_circular_track_thickness_extra_small':'2.5dp','@dimen/mtrl_progress_circular_track_thickness_medium':'4dp',
    '@dimen/mtrl_progress_circular_track_thickness_small':'3dp','@dimen/mtrl_progress_indicator_full_rounded_corner_radius':'2dp','@dimen/mtrl_progress_track_thickness':'4dp','@dimen/mtrl_shape_corner_size_large_component':'0dp',
    '@dimen/mtrl_shape_corner_size_medium_component':'4dp','@dimen/mtrl_shape_corner_size_small_component':'4dp','@dimen/mtrl_slider_halo_radius':'24dp','@dimen/mtrl_slider_label_padding':'4dp',
    '@dimen/mtrl_slider_label_radius':'13dp','@dimen/mtrl_slider_label_square_side':'26dp','@dimen/mtrl_slider_thumb_elevation':'1dp','@dimen/mtrl_slider_thumb_radius':'10dp',
    '@dimen/mtrl_slider_tick_min_spacing':'8dp','@dimen/mtrl_slider_tick_radius':'1dp','@dimen/mtrl_slider_track_height':'4dp','@dimen/mtrl_slider_track_side_padding':'16dp',
    '@dimen/mtrl_slider_widget_height':'48dp','@dimen/mtrl_snackbar_action_text_color_alpha':'0.5','@dimen/mtrl_snackbar_background_corner_radius':'4dp','@dimen/mtrl_snackbar_background_overlay_color_alpha':'0.8',
    '@dimen/mtrl_snackbar_margin':'8dp','@dimen/mtrl_snackbar_message_margin_horizontal':'8dp','@dimen/mtrl_snackbar_padding_horizontal':'8dp','@dimen/mtrl_switch_text_padding':'16dp',
    '@dimen/mtrl_switch_thumb_elevation':'4dp','@dimen/mtrl_switch_thumb_icon_size':'16dp','@dimen/mtrl_switch_thumb_size':'32dp','@dimen/mtrl_switch_track_height':'32dp',
    '@dimen/mtrl_switch_track_width':'52dp','@dimen/mtrl_textinput_box_corner_radius_medium':'4dp','@dimen/mtrl_textinput_box_corner_radius_small':'0dp','@dimen/mtrl_textinput_box_label_cutout_padding':'4dp',
    '@dimen/mtrl_textinput_box_stroke_width_default':'1dp','@dimen/mtrl_textinput_box_stroke_width_focused':'2dp','@dimen/mtrl_textinput_counter_margin_start':'16dp','@dimen/mtrl_textinput_end_icon_margin_start':'4dp',
    '@dimen/mtrl_textinput_outline_box_expanded_padding':'16dp','@dimen/mtrl_textinput_start_icon_margin_end':'4dp','@dimen/mtrl_toolbar_default_height':'56dp','@dimen/mtrl_tooltip_arrowSize':'14dp',
    '@dimen/mtrl_tooltip_cornerSize':'4dp','@dimen/mtrl_tooltip_minHeight':'32dp','@dimen/mtrl_tooltip_minWidth':'32dp','@dimen/mtrl_tooltip_padding':'12dp',
    '@dimen/mtrl_transition_shared_axis_slide_distance':'30dp',
}
BUILTIN_LIBRARY_COLORS={
    '@color/abc_primary_text_material_light':'#de000000','@color/abc_primary_text_material_dark':'#ffffffff','@color/abc_secondary_text_material_light':'#8a000000','@color/abc_secondary_text_material_dark':'#b3ffffff',
    '@color/abc_primary_text_disable_only_material_light':'#de000000','@color/abc_primary_text_disable_only_material_dark':'#ffffffff','@color/abc_hint_foreground_material_light':'#61000000','@color/abc_hint_foreground_material_dark':'#80ffffff',
    '@color/abc_background_cache_hint_selector_material_light':'#fffafafa','@color/abc_background_cache_hint_selector_material_dark':'#ff303030','@color/abc_btn_colored_borderless_text_material':'?attr/colorAccent','@color/abc_btn_colored_text_material':'#ffffffff',
    '@color/abc_color_highlight_material':'?attr/colorControlHighlight','@color/abc_decor_view_status_guard':'#ff000000','@color/abc_decor_view_status_guard_light':'#ffffffff','@color/abc_search_url_text':'#ff7fa87f',
    '@color/abc_search_url_text_normal':'#ff7fa87f','@color/abc_search_url_text_pressed':'#ff000000','@color/abc_search_url_text_selected':'#ff000000','@color/abc_tint_btn_checkable':'?attr/colorControlNormal',
    '@color/abc_tint_default':'?attr/colorControlNormal','@color/abc_tint_edittext':'?attr/colorControlNormal','@color/abc_tint_seek_thumb':'?attr/colorControlActivated','@color/abc_tint_spinner':'?attr/colorControlNormal',
    '@color/abc_tint_switch_track':'?attr/colorControlNormal','@color/material_blue_grey_800':'#ff37474f','@color/material_blue_grey_900':'#ff263238','@color/material_blue_grey_950':'#ff21272b',
    '@color/material_deep_teal_200':'#ff80cbc4','@color/material_deep_teal_500':'#ff008577','@color/material_grey_50':'#fffafafa','@color/material_grey_100':'#fff5f5f5',
    '@color/material_grey_300':'#ffe0e0e0','@color/material_grey_600':'#ff757575','@color/material_grey_800':'#ff424242','@color/material_grey_850':'#ff303030',
    '@color/material_grey_900':'#ff212121','@color/design_bottom_navigation_shadow_color':'#14000000','@color/design_dark_default_color_background':'#121212','@color/design_dark_default_color_error':'#CF6679',
    '@color/design_dark_default_color_on_background':'#FFFFFF','@color/design_dark_default_color_on_error':'#000000','@color/design_dark_default_color_on_primary':'#000000','@color/design_dark_default_color_on_secondary':'#000000',
    '@color/design_dark_default_color_on_surface':'#FFFFFF','@color/design_dark_default_color_primary':'#BA86FC','@color/design_dark_default_color_primary_dark':'#000000','@color/design_dark_default_color_primary_variant':'#3700B3',
    '@color/design_dark_default_color_secondary':'#03DAC6','@color/design_dark_default_color_secondary_variant':'#03DAC6','@color/design_dark_default_color_surface':'#121212','@color/design_default_color_background':'#FFFFFF',
    '@color/design_default_color_error':'#B00020','@color/design_default_color_on_background':'#000000','@color/design_default_color_on_error':'#FFFFFF','@color/design_default_color_on_primary':'#FFFFFF',
    '@color/design_default_color_on_secondary':'#000000','@color/design_default_color_on_surface':'#000000','@color/design_default_color_primary':'#6200EE','@color/design_default_color_primary_dark':'#3700B3',
    '@color/design_default_color_primary_variant':'#3700B3','@color/design_default_color_secondary':'#03DAC6','@color/design_default_color_secondary_variant':'#018786','@color/design_default_color_surface':'#FFFFFF',
    '@color/design_fab_shadow_end_color':'@android:color/transparent','@color/design_fab_shadow_mid_color':'#14000000','@color/design_fab_shadow_start_color':'#44000000','@color/design_fab_stroke_end_inner_color':'#0A000000',
    '@color/design_fab_stroke_end_outer_color':'#0F000000','@color/design_fab_stroke_top_inner_color':'#1AFFFFFF','@color/design_fab_stroke_top_outer_color':'#2EFFFFFF','@color/design_snackbar_background_color':'#323232',
    '@color/m3_ref_palette_black':'#ff000000','@color/m3_ref_palette_error0':'#ff000000','@color/m3_ref_palette_error10':'#ff410e0b','@color/m3_ref_palette_error100':'#ffffffff',
    '@color/m3_ref_palette_error20':'#ff601410','@color/m3_ref_palette_error30':'#ff8c1d18','@color/m3_ref_palette_error40':'#ffb3261e','@color/m3_ref_palette_error50':'#ffdc362e',
    '@color/m3_ref_palette_error60':'#ffe46962','@color/m3_ref_palette_error70':'#ffec928e','@color/m3_ref_palette_error80':'#fff2b8b5','@color/m3_ref_palette_error90':'#fff9dedc',
    '@color/m3_ref_palette_error95':'#fffceeee','@color/m3_ref_palette_error98':'#fffff8f7','@color/m3_ref_palette_error99':'#fffffbf9','@color/m3_ref_palette_neutral0':'#ff000000',
    '@color/m3_ref_palette_neutral10':'#ff1d1b20','@color/m3_ref_palette_neutral100':'#ffffffff','@color/m3_ref_palette_neutral12':'#ff211f26','@color/m3_ref_palette_neutral17':'#ff2b2930',
    '@color/m3_ref_palette_neutral20':'#ff322f35','@color/m3_ref_palette_neutral22':'#ff36343b','@color/m3_ref_palette_neutral24':'#ff3b383e','@color/m3_ref_palette_neutral30':'#ff48464c',
    '@color/m3_ref_palette_neutral4':'#ff0f0d13','@color/m3_ref_palette_neutral40':'#ff605d64','@color/m3_ref_palette_neutral50':'#ff79767d','@color/m3_ref_palette_neutral6':'#ff141218',
    '@color/m3_ref_palette_neutral60':'#ff938f96','@color/m3_ref_palette_neutral70':'#ffaea9b1','@color/m3_ref_palette_neutral80':'#ffcac5cd','@color/m3_ref_palette_neutral87':'#ffded8e1',
    '@color/m3_ref_palette_neutral90':'#ffe6e0e9','@color/m3_ref_palette_neutral92':'#ffece6f0','@color/m3_ref_palette_neutral94':'#fff3edf7','@color/m3_ref_palette_neutral95':'#fff5eff7',
    '@color/m3_ref_palette_neutral96':'#fff7f2fa','@color/m3_ref_palette_neutral98':'#fffef7ff','@color/m3_ref_palette_neutral99':'#fffffbff','@color/m3_ref_palette_neutral_variant0':'#ff000000',
    '@color/m3_ref_palette_neutral_variant10':'#ff1d1a22','@color/m3_ref_palette_neutral_variant100':'#ffffffff','@color/m3_ref_palette_neutral_variant20':'#ff322f37','@color/m3_ref_palette_neutral_variant30':'#ff49454f',
    '@color/m3_ref_palette_neutral_variant40':'#ff605d66','@color/m3_ref_palette_neutral_variant50':'#ff79747e','@color/m3_ref_palette_neutral_variant60':'#ff938f99','@color/m3_ref_palette_neutral_variant70':'#ffaea9b4',
    '@color/m3_ref_palette_neutral_variant80':'#ffcac4d0','@color/m3_ref_palette_neutral_variant90':'#ffe7e0ec','@color/m3_ref_palette_neutral_variant95':'#fff5eefa','@color/m3_ref_palette_neutral_variant98':'#fffdf7ff',
    '@color/m3_ref_palette_neutral_variant99':'#fffffbfe','@color/m3_ref_palette_primary0':'#ff000000','@color/m3_ref_palette_primary10':'#ff21005d','@color/m3_ref_palette_primary100':'#ffffffff',
    '@color/m3_ref_palette_primary20':'#ff381e72','@color/m3_ref_palette_primary30':'#ff4f378b','@color/m3_ref_palette_primary40':'#ff6750a4','@color/m3_ref_palette_primary50':'#ff7f67be',
    '@color/m3_ref_palette_primary60':'#ff9a82db','@color/m3_ref_palette_primary70':'#ffb69df8','@color/m3_ref_palette_primary80':'#ffd0bcff','@color/m3_ref_palette_primary90':'#ffeaddff',
    '@color/m3_ref_palette_primary95':'#fff6edff','@color/m3_ref_palette_primary98':'#fffef7ff','@color/m3_ref_palette_primary99':'#fffffbfe','@color/m3_ref_palette_secondary0':'#ff000000',
    '@color/m3_ref_palette_secondary10':'#ff1d192b','@color/m3_ref_palette_secondary100':'#ffffffff','@color/m3_ref_palette_secondary20':'#ff332d41','@color/m3_ref_palette_secondary30':'#ff4a4458',
    '@color/m3_ref_palette_secondary40':'#ff625b71','@color/m3_ref_palette_secondary50':'#ff7a7289','@color/m3_ref_palette_secondary60':'#ff958da5','@color/m3_ref_palette_secondary70':'#ffb0a7c0',
    '@color/m3_ref_palette_secondary80':'#ffccc2dc','@color/m3_ref_palette_secondary90':'#ffe8def8','@color/m3_ref_palette_secondary95':'#fff6edff','@color/m3_ref_palette_secondary98':'#fffef7ff',
    '@color/m3_ref_palette_secondary99':'#fffffbfe','@color/m3_ref_palette_tertiary0':'#ff000000','@color/m3_ref_palette_tertiary10':'#ff31111d','@color/m3_ref_palette_tertiary100':'#ffffffff',
    '@color/m3_ref_palette_tertiary20':'#ff492532','@color/m3_ref_palette_tertiary30':'#ff633b48','@color/m3_ref_palette_tertiary40':'#ff7d5260','@color/m3_ref_palette_tertiary50':'#ff986977',
    '@color/m3_ref_palette_tertiary60':'#ffb58392','@color/m3_ref_palette_tertiary70':'#ffd29dac','@color/m3_ref_palette_tertiary80':'#ffefb8c8','@color/m3_ref_palette_tertiary90':'#ffffd8e4',
    '@color/m3_ref_palette_tertiary95':'#ffffecf1','@color/m3_ref_palette_tertiary98':'#fffff8f8','@color/m3_ref_palette_tertiary99':'#fffffbfa','@color/m3_ref_palette_white':'#ffffffff',
    '@color/m3_sys_color_dark_background':'#ff141218','@color/m3_sys_color_dark_error':'#fff2b8b5','@color/m3_sys_color_dark_error_container':'#ff8c1d18','@color/m3_sys_color_dark_inverse_on_surface':'#ff322f35',
    '@color/m3_sys_color_dark_inverse_primary':'#ff6750a4','@color/m3_sys_color_dark_inverse_surface':'#ffe6e0e9','@color/m3_sys_color_dark_on_background':'#ffe6e0e9','@color/m3_sys_color_dark_on_error':'#ff601410',
    '@color/m3_sys_color_dark_on_error_container':'#fff9dedc','@color/m3_sys_color_dark_on_primary':'#ff381e72','@color/m3_sys_color_dark_on_primary_container':'#ffeaddff','@color/m3_sys_color_dark_on_secondary':'#ff332d41',
    '@color/m3_sys_color_dark_on_secondary_container':'#ffe8def8','@color/m3_sys_color_dark_on_surface':'#ffe6e0e9','@color/m3_sys_color_dark_on_surface_variant':'#ffcac4d0','@color/m3_sys_color_dark_on_tertiary':'#ff492532',
    '@color/m3_sys_color_dark_on_tertiary_container':'#ffffd8e4','@color/m3_sys_color_dark_outline':'#ff938f99','@color/m3_sys_color_dark_outline_variant':'#ff49454f','@color/m3_sys_color_dark_primary':'#ffd0bcff',
    '@color/m3_sys_color_dark_primary_container':'#ff4f378b','@color/m3_sys_color_dark_secondary':'#ffccc2dc','@color/m3_sys_color_dark_secondary_container':'#ff4a4458','@color/m3_sys_color_dark_surface':'#ff141218',
    '@color/m3_sys_color_dark_surface_bright':'#ff3b383e','@color/m3_sys_color_dark_surface_container':'#ff211f26','@color/m3_sys_color_dark_surface_container_high':'#ff2b2930','@color/m3_sys_color_dark_surface_container_highest':'#ff36343b',
    '@color/m3_sys_color_dark_surface_container_low':'#ff1d1b20','@color/m3_sys_color_dark_surface_container_lowest':'#ff0f0d13','@color/m3_sys_color_dark_surface_dim':'#ff141218','@color/m3_sys_color_dark_surface_variant':'#ff49454f',
    '@color/m3_sys_color_dark_tertiary':'#ffefb8c8','@color/m3_sys_color_dark_tertiary_container':'#ff633b48','@color/m3_sys_color_light_background':'#fffef7ff','@color/m3_sys_color_light_error':'#ffb3261e',
    '@color/m3_sys_color_light_error_container':'#fff9dedc','@color/m3_sys_color_light_inverse_on_surface':'#fff5eff7','@color/m3_sys_color_light_inverse_primary':'#ffd0bcff','@color/m3_sys_color_light_inverse_surface':'#ff322f35',
    '@color/m3_sys_color_light_on_background':'#ff1d1b20','@color/m3_sys_color_light_on_error':'#ffffffff','@color/m3_sys_color_light_on_error_container':'#ff410e0b','@color/m3_sys_color_light_on_primary':'#ffffffff',
    '@color/m3_sys_color_light_on_primary_container':'#ff21005d','@color/m3_sys_color_light_on_secondary':'#ffffffff','@color/m3_sys_color_light_on_secondary_container':'#ff1d192b','@color/m3_sys_color_light_on_surface':'#ff1d1b20',
    '@color/m3_sys_color_light_on_surface_variant':'#ff49454f','@color/m3_sys_color_light_on_tertiary':'#ffffffff','@color/m3_sys_color_light_on_tertiary_container':'#ff31111d','@color/m3_sys_color_light_outline':'#ff79747e',
    '@color/m3_sys_color_light_outline_variant':'#ffcac4d0','@color/m3_sys_color_light_primary':'#ff6750a4','@color/m3_sys_color_light_primary_container':'#ffeaddff','@color/m3_sys_color_light_secondary':'#ff625b71',
    '@color/m3_sys_color_light_secondary_container':'#ffe8def8','@color/m3_sys_color_light_surface':'#fffef7ff','@color/m3_sys_color_light_surface_bright':'#fffef7ff','@color/m3_sys_color_light_surface_container':'#fff3edf7',
    '@color/m3_sys_color_light_surface_container_high':'#ffece6f0','@color/m3_sys_color_light_surface_container_highest':'#ffe6e0e9','@color/m3_sys_color_light_surface_container_low':'#fff7f2fa','@color/m3_sys_color_light_surface_container_lowest':'#ffffffff',
    '@color/m3_sys_color_light_surface_dim':'#ffded8e1','@color/m3_sys_color_light_surface_variant':'#ffe7e0ec','@color/m3_sys_color_light_tertiary':'#ff7d5260','@color/m3_sys_color_light_tertiary_container':'#ffffd8e4',
    '@color/m3_sys_color_on_primary_fixed':'#ff21005d','@color/m3_sys_color_on_primary_fixed_variant':'#ff4f378b','@color/m3_sys_color_on_secondary_fixed':'#ff1d192b','@color/m3_sys_color_on_secondary_fixed_variant':'#ff4a4458',
    '@color/m3_sys_color_on_tertiary_fixed':'#ff31111d','@color/m3_sys_color_on_tertiary_fixed_variant':'#ff633b48','@color/m3_sys_color_primary_fixed':'#ffeaddff','@color/m3_sys_color_primary_fixed_dim':'#ffd0bcff',
    '@color/m3_sys_color_secondary_fixed':'#ffe8def8','@color/m3_sys_color_secondary_fixed_dim':'#ffccc2dc','@color/m3_sys_color_tertiary_fixed':'#ffffd8e4','@color/m3_sys_color_tertiary_fixed_dim':'#ffefb8c8',
    '@color/mtrl_btn_text_color_disabled':'#61000000','@color/mtrl_btn_transparent_bg_color':'#00ffffff','@color/mtrl_scrim_color':'#52000000','@color/mtrl_textinput_default_box_stroke_color':'#6B000000',
    '@color/mtrl_textinput_disabled_color':'#1F000000','@color/mtrl_textinput_filled_box_default_background_color':'#0A000000','@color/mtrl_textinput_focused_box_stroke_color':'#00000000','@color/mtrl_textinput_hovered_box_stroke_color':'#DE000000',
}
for _grp in ('primary','secondary','tertiary','neutral','neutral_variant'):
    for _tone in (0,10,20,30,40,50,60,70,80,90,95,99,100):
        _ref='@color/m3_ref_palette_'+_grp+str(_tone)
        if _ref in BUILTIN_LIBRARY_COLORS:
            BUILTIN_LIBRARY_COLORS['@color/material_dynamic_'+_grp+str(_tone)]=BUILTIN_LIBRARY_COLORS[_ref]
for _err_k in ('light_error','light_on_error','light_error_container','light_on_error_container','dark_error','dark_on_error','dark_error_container','dark_on_error_container'):
    BUILTIN_LIBRARY_COLORS['@color/material_dynamic_color_'+_err_k]=BUILTIN_LIBRARY_COLORS['@color/m3_sys_color_'+_err_k]
ANDROID_IDS={
    'text1','text2','selectedIcon','tabs','tabcontent','hint','primary','icon','icon1','icon2',
    'list','empty','content','title','summary','message','progress','secondaryProgress','background',
    'button1','button2','button3','checkbox','edit','input','home','up','widget_frame','icon_frame',
    'closeButton','custom','toggle','switch_widget','mask','candidatesArea','extractArea','inputArea',
    'inputExtractEditText','navigationBarBackground','statusBarBackground',
}
# Pinned from context/reference-repos/material-android/lib/java/com/google/android/material/typography/res/values/{tokens,styles}.xml
# and context/reference-sources/android-framework/res/values/{styles_material,dimens_material}.xml
_M3_TYPESCALE={
    'DisplayLarge':{A+'textSize':'57sp',A+'letterSpacing':'-0.00438596',A+'textAllCaps':'false'},
    'DisplayMedium':{A+'textSize':'45sp',A+'letterSpacing':'0',A+'textAllCaps':'false'},
    'DisplaySmall':{A+'textSize':'36sp',A+'letterSpacing':'0',A+'textAllCaps':'false'},
    'HeadlineLarge':{A+'textSize':'32sp',A+'letterSpacing':'0',A+'textAllCaps':'false'},
    'HeadlineMedium':{A+'textSize':'28sp',A+'letterSpacing':'0',A+'textAllCaps':'false'},
    'HeadlineSmall':{A+'textSize':'24sp',A+'letterSpacing':'0',A+'textAllCaps':'false'},
    'TitleLarge':{A+'textSize':'22sp',A+'letterSpacing':'0',A+'textAllCaps':'false'},
    'TitleMedium':{A+'textSize':'16sp',A+'letterSpacing':'0.009375',A+'textAllCaps':'false'},
    'TitleSmall':{A+'textSize':'14sp',A+'letterSpacing':'0.00714286',A+'textAllCaps':'false'},
    'BodyLarge':{A+'textSize':'16sp',A+'letterSpacing':'0.03125',A+'textAllCaps':'false'},
    'BodyMedium':{A+'textSize':'14sp',A+'letterSpacing':'0.01785714',A+'textAllCaps':'false'},
    'BodySmall':{A+'textSize':'12sp',A+'letterSpacing':'0.03333333',A+'textAllCaps':'false'},
    'LabelLarge':{A+'textSize':'14sp',A+'letterSpacing':'0.00714286',A+'textAllCaps':'false'},
    'LabelMedium':{A+'textSize':'12sp',A+'letterSpacing':'0.04166667',A+'textAllCaps':'false'},
    'LabelSmall':{A+'textSize':'11sp',A+'letterSpacing':'0.04545455',A+'textAllCaps':'false'},
}
_M3_EMPHASIZED_BOLD={'TitleMedium','TitleSmall','LabelLarge','LabelMedium','LabelSmall'}
_MDC_TYPESCALE={
    'Headline1':{A+'textSize':'96sp',A+'letterSpacing':'-0.015625',A+'textStyle':'normal',A+'textAllCaps':'false'},
    'Headline2':{A+'textSize':'60sp',A+'letterSpacing':'-0.00833333333',A+'textStyle':'normal',A+'textAllCaps':'false'},
    'Headline3':{A+'textSize':'48sp',A+'letterSpacing':'0',A+'textStyle':'normal',A+'textAllCaps':'false'},
    'Headline4':{A+'textSize':'34sp',A+'letterSpacing':'0.00735294118',A+'textStyle':'normal',A+'textAllCaps':'false'},
    'Headline5':{A+'textSize':'24sp',A+'letterSpacing':'0',A+'textStyle':'normal',A+'textAllCaps':'false'},
    'Headline6':{A+'textSize':'20sp',A+'letterSpacing':'0.0125',A+'textStyle':'normal',A+'textAllCaps':'false'},
    'Subtitle1':{A+'textSize':'16sp',A+'letterSpacing':'0.009375',A+'textStyle':'normal',A+'textAllCaps':'false'},
    'Subtitle2':{A+'textSize':'14sp',A+'letterSpacing':'0.00714285714',A+'textStyle':'normal',A+'textAllCaps':'false'},
    'Body1':{A+'textSize':'16sp',A+'letterSpacing':'0.03125',A+'textStyle':'normal',A+'textAllCaps':'false'},
    'Body2':{A+'textSize':'14sp',A+'letterSpacing':'0.0178571429',A+'textStyle':'normal',A+'textAllCaps':'false'},
    'Caption':{A+'textSize':'12sp',A+'letterSpacing':'0.0333333333',A+'textStyle':'normal',A+'textAllCaps':'false'},
    'Button':{A+'textSize':'14sp',A+'letterSpacing':'0.0892857143',A+'textStyle':'normal',A+'textAllCaps':'true'},
    'Overline':{A+'textSize':'10sp',A+'letterSpacing':'0.166666667',A+'textStyle':'normal',A+'textAllCaps':'true'},
    'Chip':{A+'textSize':'14sp',A+'letterSpacing':'0.0178571429',A+'textStyle':'normal',A+'textAllCaps':'false'},
    'Tab':{A+'textSize':'14sp',A+'letterSpacing':'0.0892857143',A+'textStyle':'normal',A+'textAllCaps':'true'},
    'Tooltip':{A+'textSize':'12sp',A+'letterSpacing':'0',A+'textStyle':'normal',A+'textAllCaps':'false'},
    'Badge':{A+'textSize':'10sp',A+'letterSpacing':'0',A+'textStyle':'bold',A+'textAllCaps':'false'},
}
_FRAMEWORK_TEXT_APPEARANCE={
    '':{A+'textSize':'14sp',A+'textColor':'?attr/textColorPrimary'},
    'Display4':{A+'textSize':'112sp',A+'textColor':'?attr/textColorSecondary'},
    'Display3':{A+'textSize':'56sp',A+'textColor':'?attr/textColorSecondary'},
    'Display2':{A+'textSize':'45sp',A+'textColor':'?attr/textColorSecondary'},
    'Display1':{A+'textSize':'34sp',A+'textColor':'?attr/textColorSecondary'},
    'Headline':{A+'textSize':'24sp',A+'textColor':'?attr/textColorPrimary'},
    'Title':{A+'textSize':'20sp',A+'textColor':'?attr/textColorPrimary'},
    'Title.Inverse':{A+'textSize':'20sp',A+'textColor':'?attr/textColorPrimaryInverse'},
    'Subhead':{A+'textSize':'16sp',A+'textColor':'?attr/textColorPrimary'},
    'Subhead.Inverse':{A+'textSize':'16sp',A+'textColor':'?attr/textColorPrimaryInverse'},
    'Body2':{A+'textSize':'14sp',A+'textColor':'?attr/textColorPrimary'},
    'Body1':{A+'textSize':'14sp',A+'textColor':'?attr/textColorPrimary'},
    'Caption':{A+'textSize':'12sp',A+'textColor':'?attr/textColorSecondary'},
    'Menu':{A+'textSize':'16sp'},
    'Menu.Inverse':{A+'textSize':'16sp',A+'textColor':'?attr/textColorSecondaryInverse'},
    'Button':{A+'textSize':'14sp',A+'textAllCaps':'true',A+'textColor':'?attr/textColorPrimary'},
    'Inverse':{A+'textSize':'14sp',A+'textColor':'?attr/textColorPrimaryInverse'},
    'Large':{A+'textSize':'22sp',A+'textColor':'?attr/textColorPrimary'},
    'Large.Inverse':{A+'textSize':'22sp',A+'textColor':'?attr/textColorPrimaryInverse'},
    'Medium':{A+'textSize':'18sp',A+'textColor':'?attr/textColorSecondary'},
    'Medium.Inverse':{A+'textSize':'18sp',A+'textColor':'?attr/textColorSecondaryInverse'},
    'Small':{A+'textSize':'14sp',A+'textColor':'?attr/textColorTertiary'},
    'Small.Inverse':{A+'textSize':'14sp',A+'textColor':'?attr/textColorTertiaryInverse'},
    'SearchResult.Title':{A+'textSize':'18sp',A+'textColor':'?attr/textColorPrimary'},
    'SearchResult.Subtitle':{A+'textSize':'14sp',A+'textColor':'?attr/textColorSecondary'},
    'WindowTitle':{A+'textSize':'20sp',A+'textColor':'?attr/textColorPrimary'},
    'DialogWindowTitle':{A+'textSize':'20sp',A+'textColor':'?attr/textColorPrimary'},
    'Widget.Button':{A+'textSize':'14sp',A+'textAllCaps':'true',A+'textColor':'?attr/textColorPrimary'},
    'Widget.Button.Inverse':{A+'textSize':'14sp',A+'textAllCaps':'true',A+'textColor':'?attr/textColorPrimaryInverse'},
    'Widget.EditText':{A+'textSize':'16sp',A+'textColor':'?attr/textColorPrimary'},
    'Widget.Switch':{A+'textSize':'14sp',A+'textAllCaps':'true',A+'textColor':'?attr/textColorPrimary'},
    'Widget.TextView':{A+'textSize':'14sp',A+'textColor':'?attr/textColorPrimary'},
    'Widget.TextView.PopupMenu':{A+'textSize':'16sp'},
    'Widget.TextView.SpinnerItem':{A+'textSize':'16sp'},
    'Widget.DropDownItem':{A+'textSize':'16sp',A+'textColor':'?attr/textColorPrimary'},
    'Widget.ActionBar.Title':{A+'textSize':'20dp',A+'textColor':'?attr/textColorPrimary'},
    'Widget.ActionBar.Title.Inverse':{A+'textSize':'20dp',A+'textColor':'?attr/textColorPrimaryInverse'},
    'Widget.ActionBar.Subtitle':{A+'textSize':'16dp',A+'textColor':'?attr/textColorSecondary'},
    'Widget.ActionBar.Subtitle.Inverse':{A+'textSize':'16dp',A+'textColor':'?attr/textColorSecondaryInverse'},
    'Widget.ActionMode.Title':{A+'textSize':'20dp',A+'textColor':'?attr/textColorPrimary'},
    'Widget.ActionMode.Subtitle':{A+'textSize':'16dp',A+'textColor':'?attr/textColorSecondary'},
    'Widget.Toolbar.Title':{A+'textSize':'20dp',A+'textColor':'?attr/textColorPrimary'},
    'Widget.Toolbar.Subtitle':{A+'textSize':'16dp',A+'textColor':'?attr/textColorSecondary'},
    'Widget.PopupMenu.Large':{A+'textSize':'16sp'},
    'Widget.PopupMenu.Small':{A+'textSize':'16sp'},
    'Widget.PopupMenu.Header':{A+'textSize':'14sp',A+'textColor':'?attr/colorAccent'},
}
BUILTIN_TEXT_APPEARANCES={}
for _k,_v in _M3_TYPESCALE.items():
    BUILTIN_TEXT_APPEARANCES['TextAppearance.Material3.'+_k]=dict(_v)
    BUILTIN_TEXT_APPEARANCES['TextAppearance.M3.Sys.Typescale.'+_k]=dict(_v)
    _emp=dict(_v)
    if _k=='DisplayLarge':_emp[A+'letterSpacing']='0'
    if _k in _M3_EMPHASIZED_BOLD:_emp[A+'textStyle']='bold'
    BUILTIN_TEXT_APPEARANCES['TextAppearance.Material3.'+_k+'.Emphasized']=_emp
    BUILTIN_TEXT_APPEARANCES['TextAppearance.M3.Sys.Typescale.'+_k+'.Emphasized']=dict(_emp)
BUILTIN_TEXT_APPEARANCES['TextAppearance.Material3.ActionBar.Title']={A+'textSize':'22sp',A+'textStyle':'normal',A+'textAllCaps':'false'}
BUILTIN_TEXT_APPEARANCES['TextAppearance.Material3.ActionBar.Subtitle']={A+'textSize':'16sp',A+'textStyle':'normal',A+'textAllCaps':'false'}
for _k,_v in _MDC_TYPESCALE.items():
    BUILTIN_TEXT_APPEARANCES['TextAppearance.MaterialComponents.'+_k]=dict(_v)
    BUILTIN_TEXT_APPEARANCES['Base.TextAppearance.MaterialComponents.'+_k]=dict(_v)
for _prefix in ('TextAppearance','TextAppearance.Material','TextAppearance.AppCompat','Base.TextAppearance.AppCompat','TextAppearance.Holo','TextAppearance.DeviceDefault','TextAppearance.Design'):
    for _k,_v in _FRAMEWORK_TEXT_APPEARANCE.items():
        _full=_prefix+('.'+_k if _k else '')
        BUILTIN_TEXT_APPEARANCES.setdefault(_full,dict(_v))
THEME_STYLE_REFS={
    'textAppearance':'@style/TextAppearance.Material',
    'textAppearanceDisplayLarge':'@style/TextAppearance.Material3.DisplayLarge',
    'textAppearanceDisplayMedium':'@style/TextAppearance.Material3.DisplayMedium',
    'textAppearanceDisplaySmall':'@style/TextAppearance.Material3.DisplaySmall',
    'textAppearanceHeadlineLarge':'@style/TextAppearance.Material3.HeadlineLarge',
    'textAppearanceHeadlineMedium':'@style/TextAppearance.Material3.HeadlineMedium',
    'textAppearanceHeadlineSmall':'@style/TextAppearance.Material3.HeadlineSmall',
    'textAppearanceTitleLarge':'@style/TextAppearance.Material3.TitleLarge',
    'textAppearanceTitleMedium':'@style/TextAppearance.Material3.TitleMedium',
    'textAppearanceTitleSmall':'@style/TextAppearance.Material3.TitleSmall',
    'textAppearanceBodyLarge':'@style/TextAppearance.Material3.BodyLarge',
    'textAppearanceBodyMedium':'@style/TextAppearance.Material3.BodyMedium',
    'textAppearanceBodySmall':'@style/TextAppearance.Material3.BodySmall',
    'textAppearanceLabelLarge':'@style/TextAppearance.Material3.LabelLarge',
    'textAppearanceLabelMedium':'@style/TextAppearance.Material3.LabelMedium',
    'textAppearanceLabelSmall':'@style/TextAppearance.Material3.LabelSmall',
    'textAppearanceDisplayLargeEmphasized':'@style/TextAppearance.Material3.DisplayLarge.Emphasized',
    'textAppearanceDisplayMediumEmphasized':'@style/TextAppearance.Material3.DisplayMedium.Emphasized',
    'textAppearanceDisplaySmallEmphasized':'@style/TextAppearance.Material3.DisplaySmall.Emphasized',
    'textAppearanceHeadlineLargeEmphasized':'@style/TextAppearance.Material3.HeadlineLarge.Emphasized',
    'textAppearanceHeadlineMediumEmphasized':'@style/TextAppearance.Material3.HeadlineMedium.Emphasized',
    'textAppearanceHeadlineSmallEmphasized':'@style/TextAppearance.Material3.HeadlineSmall.Emphasized',
    'textAppearanceTitleLargeEmphasized':'@style/TextAppearance.Material3.TitleLarge.Emphasized',
    'textAppearanceTitleMediumEmphasized':'@style/TextAppearance.Material3.TitleMedium.Emphasized',
    'textAppearanceTitleSmallEmphasized':'@style/TextAppearance.Material3.TitleSmall.Emphasized',
    'textAppearanceBodyLargeEmphasized':'@style/TextAppearance.Material3.BodyLarge.Emphasized',
    'textAppearanceBodyMediumEmphasized':'@style/TextAppearance.Material3.BodyMedium.Emphasized',
    'textAppearanceBodySmallEmphasized':'@style/TextAppearance.Material3.BodySmall.Emphasized',
    'textAppearanceLabelLargeEmphasized':'@style/TextAppearance.Material3.LabelLarge.Emphasized',
    'textAppearanceLabelMediumEmphasized':'@style/TextAppearance.Material3.LabelMedium.Emphasized',
    'textAppearanceLabelSmallEmphasized':'@style/TextAppearance.Material3.LabelSmall.Emphasized',
    'textAppearanceHeadline1':'@style/TextAppearance.MaterialComponents.Headline1',
    'textAppearanceHeadline2':'@style/TextAppearance.MaterialComponents.Headline2',
    'textAppearanceHeadline3':'@style/TextAppearance.MaterialComponents.Headline3',
    'textAppearanceHeadline4':'@style/TextAppearance.MaterialComponents.Headline4',
    'textAppearanceHeadline5':'@style/TextAppearance.MaterialComponents.Headline5',
    'textAppearanceHeadline6':'@style/TextAppearance.MaterialComponents.Headline6',
    'textAppearanceSubtitle1':'@style/TextAppearance.MaterialComponents.Subtitle1',
    'textAppearanceSubtitle2':'@style/TextAppearance.MaterialComponents.Subtitle2',
    'textAppearanceBody1':'@style/TextAppearance.MaterialComponents.Body1',
    'textAppearanceBody2':'@style/TextAppearance.MaterialComponents.Body2',
    'textAppearanceCaption':'@style/TextAppearance.MaterialComponents.Caption',
    'textAppearanceButton':'@style/TextAppearance.MaterialComponents.Button',
    'textAppearanceOverline':'@style/TextAppearance.MaterialComponents.Overline',
    'textAppearanceListItem':'@style/TextAppearance.Material.Subhead',
    'textAppearanceListItemSecondary':'@style/TextAppearance.Material.Body1',
    'textAppearanceListItemSmall':'@style/TextAppearance.Material.Body1',
    'textAppearanceLarge':'@style/TextAppearance.Material.Large',
    'textAppearanceMedium':'@style/TextAppearance.Material.Medium',
    'textAppearanceSmall':'@style/TextAppearance.Material.Small',
    'textAppearanceInverse':'@style/TextAppearance.Material.Inverse',
    'textAppearanceLargeInverse':'@style/TextAppearance.Material.Large.Inverse',
    'textAppearanceMediumInverse':'@style/TextAppearance.Material.Medium.Inverse',
    'textAppearanceSmallInverse':'@style/TextAppearance.Material.Small.Inverse',
    'textAppearanceSearchResultTitle':'@style/TextAppearance.Material.SearchResult.Title',
    'textAppearanceSearchResultSubtitle':'@style/TextAppearance.Material.SearchResult.Subtitle',
    'textAppearancePopupMenuHeader':'@style/TextAppearance.Material.Widget.PopupMenu.Header',
    'textAppearanceLargePopupMenu':'@style/TextAppearance.Material.Widget.PopupMenu.Large',
    'textAppearanceSmallPopupMenu':'@style/TextAppearance.Material.Widget.PopupMenu.Small',
    'materialButtonStyle':'@style/Widget.Material3.Button',
    'materialButtonOutlinedStyle':'@style/Widget.Material3.Button.OutlinedButton',
    'materialButtonTonalStyle':'@style/Widget.Material3.Button.TonalButton',
    'materialButtonTextStyle':'@style/Widget.Material3.Button.TextButton',
    'materialButtonElevatedStyle':'@style/Widget.Material3.Button.ElevatedButton',
    'materialIconButtonStyle':'@style/Widget.Material3.Button.IconButton',
    'materialIconButtonFilledStyle':'@style/Widget.Material3.Button.IconButton.Filled',
    'materialIconButtonFilledTonalStyle':'@style/Widget.Material3.Button.IconButton.Filled.Tonal',
    'materialIconButtonOutlinedStyle':'@style/Widget.Material3.Button.IconButton.Outlined',
    'borderlessButtonStyle':'@style/Widget.AppCompat.Button.Borderless',
    'buttonBarButtonStyle':'@style/Widget.AppCompat.Button.Borderless',
    'buttonBarPositiveButtonStyle':'@style/Widget.AppCompat.Button.Borderless',
    'buttonBarNegativeButtonStyle':'@style/Widget.AppCompat.Button.Borderless',
    'buttonBarNeutralButtonStyle':'@style/Widget.AppCompat.Button.Borderless',
    'buttonBarStyle':'@style/Widget.AppCompat.ButtonBar',
    'buttonStyle':'@style/Widget.AppCompat.Button',
    'toolbarStyle':'@style/Widget.AppCompat.Toolbar',
    'actionBarStyle':'@style/Widget.AppCompat.ActionBar',
    'progressBarStyle':'@style/Widget.Material.ProgressBar',
    'progressBarStyleSmall':'@style/Widget.Material.ProgressBar.Small',
    'progressBarStyleLarge':'@style/Widget.Material.ProgressBar.Large',
    'progressBarStyleInverse':'@style/Widget.Material.ProgressBar',
    'progressBarStyleSmallTitle':'@style/Widget.Material.ProgressBar.Small',
    'progressBarStyleHorizontal':'@style/Widget.Material.ProgressBar.Horizontal',
    'linearProgressIndicatorStyle':'@style/Widget.Material3.LinearProgressIndicator',
    'circularProgressIndicatorStyle':'@style/Widget.Material3.CircularProgressIndicator',
    'linearWavyProgressIndicatorStyle':'@style/Widget.Material3Expressive.LinearProgressIndicator.Wavy',
    'circularWavyProgressIndicatorStyle':'@style/Widget.Material3Expressive.CircularProgressIndicator.Wavy',
    'loadingIndicatorStyle':'@style/Widget.Material3.LoadingIndicator',
    'loadingIndicatorContainedStyle':'@style/Widget.Material3.LoadingIndicator.Contained',
    'bottomSheetStyle':'@style/Widget.Material3.BottomSheet',
    'bottomSheetDragHandleStyle':'@style/Widget.Material3.BottomSheet.DragHandle',
    'sideSheetStyle':'@style/Widget.Material3.SideSheet',
    'sideSheetModalStyle':'@style/Widget.Material3.SideSheet.Modal',
    'materialAlertDialogTheme':'@style/MaterialAlertDialog.Material3',
    'materialAlertDialogStyle':'@style/MaterialAlertDialog.Material3',
    'snackbarStyle':'@style/Widget.Material3.Snackbar',
    'floatingToolbarStyle':'@style/Widget.Material3.FloatingToolbar',
    'floatingToolbarVibrantStyle':'@style/Widget.Material3.FloatingToolbar.Vibrant',
    'materialButtonGroupStyle':'@style/Widget.Material3Expressive.MaterialButtonGroup',
    'materialSplitButtonStyle':'@style/Widget.Material3.MaterialSplitButton',
    'materialSplitButtonLeadingFilledStyle':'@style/Widget.Material3Expressive.Button',
    'materialSplitButtonLeadingFilledTonalStyle':'@style/Widget.Material3Expressive.Button.TonalButton',
    'materialSplitButtonIconFilledStyle':'@style/Widget.Material3Expressive.Button.IconButton.Filled',
    'materialSplitButtonIconFilledTonalStyle':'@style/Widget.Material3Expressive.Button.IconButton.Tonal',
    'checkboxStyle':'@style/Widget.Material3.CompoundButton.CheckBox',
    'radioButtonStyle':'@style/Widget.Material3.CompoundButton.RadioButton',
    'editTextStyle':'@style/Widget.AppCompat.EditText',
    'textInputStyle':'@style/Widget.Material3.TextInputLayout.FilledBox',
    'textInputOutlinedStyle':'@style/Widget.Material3.TextInputLayout.OutlinedBox',
    'textInputFilledStyle':'@style/Widget.Material3.TextInputLayout.FilledBox',
    'floatingActionButtonStyle':'@style/Widget.Material3.FloatingActionButton.Primary',
    'extendedFloatingActionButtonStyle':'@style/Widget.Material3.ExtendedFloatingActionButton.Primary',
    'chipStyle':'@style/Widget.Material3.Chip.Assist',
    'chipGroupStyle':'@style/Widget.Material3.ChipGroup',
    'tabStyle':'@style/Widget.Material3.TabLayout',
    'tabSecondaryStyle':'@style/Widget.Material3.TabLayout.Secondary',
    'bottomNavigationStyle':'@style/Widget.Material3.BottomNavigationView',
    'navigationRailStyle':'@style/Widget.Material3.NavigationRailView',
    'navigationViewStyle':'@style/Widget.Material3.NavigationView',
    'materialSearchBarStyle':'@style/Widget.Material3.SearchBar',
    'materialSearchViewStyle':'@style/Widget.Material3.SearchView',
    'materialButtonToggleGroupStyle':'@style/Widget.Material3.MaterialButtonToggleGroup',
    'bottomAppBarStyle':'@style/Widget.Material3.BottomAppBar',
    'appBarLayoutStyle':'@style/Widget.Material3.AppBarLayout',
    'collapsingToolbarLayoutStyle':'@style/Widget.Material3.CollapsingToolbar',
    'collapsingToolbarLayoutMediumStyle':'@style/Widget.Material3.CollapsingToolbar.Medium',
    'collapsingToolbarLayoutLargeStyle':'@style/Widget.Material3.CollapsingToolbar.Large',
    'sliderStyle':'@style/Widget.Material3.Slider',
    'materialSwitchStyle':'@style/Widget.Material3.CompoundButton.MaterialSwitch',
    'switchStyle':'@style/Widget.AppCompat.CompoundButton.Switch',
    'spinnerStyle':'@style/Widget.AppCompat.Spinner',
    'dropDownSpinnerStyle':'@style/Widget.AppCompat.Spinner.DropDown',
    'ratingBarStyle':'@style/Widget.AppCompat.RatingBar',
    'ratingBarStyleSmall':'@style/Widget.AppCompat.RatingBar.Small',
    'ratingBarStyleIndicator':'@style/Widget.AppCompat.RatingBar.Indicator',
    'listSeparatorTextViewStyle':{A+'textSize':'14sp',A+'textStyle':'bold'},
    'shapeAppearanceCornerExtraSmall':'@style/ShapeAppearance.Material3.Corner.ExtraSmall',
    'shapeAppearanceCornerSmall':'@style/ShapeAppearance.Material3.Corner.Small',
    'shapeAppearanceCornerMedium':'@style/ShapeAppearance.Material3.Corner.Medium',
    'shapeAppearanceCornerLarge':'@style/ShapeAppearance.Material3.Corner.Large',
    'shapeAppearanceCornerLargeIncreased':'@style/ShapeAppearance.Material3.Corner.LargeIncreased',
    'shapeAppearanceCornerExtraLarge':'@style/ShapeAppearance.Material3.Corner.ExtraLarge',
    'shapeAppearanceCornerExtraLargeIncreased':'@style/ShapeAppearance.Material3.Corner.ExtraLargeIncreased',
    'shapeAppearanceCornerExtraExtraLarge':'@style/ShapeAppearance.Material3.Corner.ExtraExtraLarge',
    'shapeAppearanceCornerFull':'@style/ShapeAppearance.Material3.Corner.Full',
    'shapeAppearanceSmallComponent':'@style/ShapeAppearance.Material3.SmallComponent',
    'shapeAppearanceMediumComponent':'@style/ShapeAppearance.Material3.MediumComponent',
    'shapeAppearanceLargeComponent':'@style/ShapeAppearance.Material3.LargeComponent',
}
# Pinned from context/reference-repos/material-android/lib/java/com/google/android/material/shape/res/values/{tokens,styles,dimens}.xml
_M3_SHAPE_CORNERS={
    'None':'0dp','ExtraSmall':'4dp','Small':'8dp','Medium':'12dp','Large':'16dp',
    'LargeIncreased':'20dp','ExtraLarge':'28dp','ExtraLargeIncreased':'32dp','ExtraExtraLarge':'48dp','Full':'50%',
}
BUILTIN_SHAPE_APPEARANCES={
    'ShapeAppearance.Material3':{'cornerFamily':'rounded'},
    'ShapeAppearance.Material3.SmallComponent':{'cornerFamily':'rounded','cornerSize':'4dp'},
    'ShapeAppearance.Material3.MediumComponent':{'cornerFamily':'rounded','cornerSize':'8dp'},
    'ShapeAppearance.Material3.LargeComponent':{'cornerFamily':'rounded','cornerSize':'8dp'},
    'ShapeAppearance.MaterialComponents':{'cornerFamily':'rounded'},
    'ShapeAppearance.MaterialComponents.SmallComponent':{'cornerFamily':'rounded','cornerSize':'4dp'},
    'ShapeAppearance.MaterialComponents.MediumComponent':{'cornerFamily':'rounded','cornerSize':'4dp'},
    'ShapeAppearance.MaterialComponents.LargeComponent':{'cornerFamily':'rounded','cornerSize':'0dp'},
    'ShapeAppearanceOverlay.Material3.Corner.Top':{'cornerSizeBottomLeft':'0dp','cornerSizeBottomRight':'0dp'},
    'ShapeAppearanceOverlay.Material3.Corner.Bottom':{'cornerSizeTopLeft':'0dp','cornerSizeTopRight':'0dp'},
    'ShapeAppearanceOverlay.Material3.Corner.Left':{'cornerSizeTopRight':'0dp','cornerSizeBottomRight':'0dp'},
    'ShapeAppearanceOverlay.Material3.Corner.Right':{'cornerSizeTopLeft':'0dp','cornerSizeBottomLeft':'0dp'},
}
for _sk,_sv in _M3_SHAPE_CORNERS.items():
    BUILTIN_SHAPE_APPEARANCES['ShapeAppearance.Material3.Corner.'+_sk]={'cornerFamily':'rounded','cornerSize':_sv}
    BUILTIN_SHAPE_APPEARANCES['ShapeAppearance.M3.Sys.Shape.Corner.'+_sk]={'cornerFamily':'rounded','cornerSize':_sv}
BITMAP_GRAVITY_TOKENS={'top','bottom','left','right','start','end','center_vertical','fill_vertical','center_horizontal','fill_horizontal','center','fill','clip_vertical','clip_horizontal'}
BUILTIN_WIDGET_STYLES={
    'Widget.Material3.Button':{A+'backgroundTint':'?attr/colorPrimary',A+'textColor':'?attr/colorOnPrimary'},
    'Widget.Material3.Button.OutlinedButton':{A+'background':'#00000000',A+'textColor':'?attr/colorPrimary',APP+'strokeColor':'?attr/colorOutline',APP+'strokeWidth':'1dp'},
    'Widget.Material3.Button.TextButton':{A+'background':'#00000000',A+'textColor':'?attr/colorPrimary'},
    'Widget.Material3.Button.TonalButton':{A+'backgroundTint':'?attr/colorSecondaryContainer',A+'textColor':'?attr/colorOnSecondaryContainer'},
    'Widget.Material3.Button.ElevatedButton':{A+'backgroundTint':'?attr/colorSurfaceContainerLow',A+'textColor':'?attr/colorPrimary',A+'elevation':'1dp'},
    'Widget.Material3.Button.IconButton':{A+'background':'#00000000'},
    'Widget.Material3.Button.IconButton.Filled':{A+'backgroundTint':'?attr/colorPrimary'},
    'Widget.Material3.Button.IconButton.Filled.Tonal':{A+'backgroundTint':'?attr/colorSecondaryContainer'},
    'Widget.Material3.Button.IconButton.Outlined':{A+'background':'#00000000',APP+'strokeColor':'?attr/colorOutline',APP+'strokeWidth':'1dp'},
    'Widget.MaterialComponents.Button':{A+'backgroundTint':'?attr/colorPrimary',A+'textColor':'?attr/colorOnPrimary'},
    'Widget.MaterialComponents.Button.UnelevatedButton':{A+'backgroundTint':'?attr/colorPrimary',A+'textColor':'?attr/colorOnPrimary'},
    'Widget.MaterialComponents.Button.OutlinedButton':{A+'background':'#00000000',A+'textColor':'?attr/colorPrimary',APP+'strokeColor':'?attr/colorOutline',APP+'strokeWidth':'1dp'},
    'Widget.MaterialComponents.Button.TextButton':{A+'background':'#00000000',A+'textColor':'?attr/colorPrimary'},
    'Widget.AppCompat.Button.Borderless':{A+'background':'#00000000'},
    'Widget.AppCompat.Button.Borderless.Colored':{A+'background':'#00000000',A+'textColor':'?attr/colorAccent'},
    'Widget.Material.Button.Borderless':{A+'background':'#00000000'},
    'Widget.Material.Button.Borderless.Colored':{A+'background':'#00000000',A+'textColor':'?attr/colorAccent'},
    'Widget.ProgressBar.Horizontal':{A+'orientation':'horizontal'},
    'Widget.Material.ProgressBar.Horizontal':{A+'orientation':'horizontal'},
    'Widget.Holo.ProgressBar.Horizontal':{A+'orientation':'horizontal'},
    'Widget.AppCompat.ProgressBar.Horizontal':{A+'orientation':'horizontal'},
    'Base.Widget.AppCompat.ProgressBar.Horizontal':{A+'orientation':'horizontal'},
    'Widget.Material3.LinearProgressIndicator':{A+'orientation':'horizontal'},
    'Widget.MaterialComponents.LinearProgressIndicator':{A+'orientation':'horizontal'},
    'Widget.Material3Expressive.LinearProgressIndicator':{A+'orientation':'horizontal'},
    'Widget.Material3Expressive.LinearProgressIndicator.Wavy':{A+'orientation':'horizontal'},
}
BUILTIN_STYLE_PREFIXES=(
    'Widget.Material3.','Base.Widget.Material3.','Widget.Material3Expressive.','Base.Widget.Material3Expressive.',
    'MaterialAlertDialog.','Base.MaterialAlertDialog.',
    'Widget.MaterialComponents.','Base.Widget.MaterialComponents.',
    'Widget.AppCompat.','Base.Widget.AppCompat.','Base.Widget.','Widget.Design.','Widget.Support.',
    'Widget.Material.','Widget.Holo.','Widget.DeviceDefault.',
    'ThemeOverlay.','Base.ThemeOverlay.','Theme.Material3.','Theme.MaterialComponents.','Theme.AppCompat.',
    'Base.Theme.Material3.','Base.Theme.MaterialComponents.','Base.Theme.AppCompat.','Theme.Material.','Theme.Holo.','Theme.DeviceDefault.',
    'ShapeAppearance.','ShapeAppearanceOverlay.','PreferenceThemeOverlay.',
    'TextAppearance.Material3.','TextAppearance.M3.','TextAppearance.MaterialComponents.','Base.TextAppearance.MaterialComponents.',
    'TextAppearance.AppCompat.','Base.TextAppearance.AppCompat.','TextAppearance.Material.','TextAppearance.Holo.','TextAppearance.DeviceDefault.','TextAppearance.Design.',
)
BUILTIN_STYLE_EXACT={
    'Widget.Material3','Widget.MaterialComponents','Widget.AppCompat','Widget.Material','Widget.Holo','Widget.DeviceDefault',
    'ThemeOverlay.AppCompat','ThemeOverlay.Material3','ThemeOverlay.MaterialComponents',
    'Theme.AppCompat','Theme.Material3','Theme.MaterialComponents','Theme.Material','Theme.Holo','Theme.DeviceDefault',
    'ShapeAppearance.MaterialComponents','ShapeAppearance.Material3',
}
STYLE_UNPREFIXED_TO_ANDROID={
    'background','backgroundTint','elevation','textColor','textSize','textStyle','textAllCaps','fontFamily','gravity','hint','includeFontPadding',
    'minHeight','minWidth','padding','paddingLeft','paddingTop','paddingRight','paddingBottom','paddingStart','paddingEnd',
    'paddingHorizontal','paddingVertical','layout_width','layout_height','layout_gravity','layout_weight',
    'layout_margin','layout_marginLeft','layout_marginTop','layout_marginRight','layout_marginBottom','layout_marginStart','layout_marginEnd',
    'layout_marginHorizontal','layout_marginVertical','orientation','visibility','alpha','clickable','enabled','focusable',
}
STYLE_IGNORED_ANDROID_ATTRS={
    'shadowRadius','shadowColor','shadowDx','shadowDy','capitalize','autoText','textColorPrimary','textColorSecondary',
    'textColorTertiary','textColorHighlight','windowBackground','windowNoTitle','windowActionBar','windowFullscreen',
    'windowContentOverlay','windowIsTranslucent','windowLayoutInDisplayCutoutMode','navigationBarColor','statusBarColor',
    'alertDialogTheme','colorBackground','listDivider','stateListAnimator','textCursorDrawable','fadingEdge','bufferType',
    'textFontWeight','fontFeatureSettings','elegantTextHeight',
}
STYLE_IGNORED_APP_ATTRS={
    'popupTheme','titleTextStyle','subtitleTextStyle','firstBaselineToTopHeight','lastBaselineToBottomHeight',
    'theme_type','colorPrimary','colorPrimaryDark','colorAccent','colorSecondary','colorSurface','colorOnSurface','colorOnPrimary',
    'colorControlActivated','colorControlHighlight','colorControlNormal','windowActionBar','windowNoTitle','windowActionModeOverlay',
    'actionBarPopupTheme','actionBarStyle','actionBarTabBarStyle','actionModeBackground','alertDialogTheme','preferenceTheme',
    'textAllCaps','enforceTextAppearance','enforceMaterialTheme','ensureMinTouchTargetSize','useMaterialThemeColors',
}
TAG_ALIASES={
    'com.google.android.material.textview.MaterialTextView':'TextView',
    'androidx.appcompat.widget.AppCompatTextView':'TextView',
    'androidx.appcompat.widget.AppCompatImageView':'ImageView',
    'androidx.appcompat.widget.AppCompatImageButton':'ImageButton',
    'androidx.appcompat.widget.AppCompatEditText':'EditText',
    'com.google.android.material.textfield.TextInputEditText':'EditText',
    'com.google.android.material.button.MaterialButton':'Button',
    'androidx.appcompat.widget.AppCompatButton':'Button',
    'com.google.android.material.floatingactionbutton.FloatingActionButton':'ImageButton',
    'com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton':'Button',
    'com.google.android.material.appbar.MaterialToolbar':'Toolbar',
    'com.google.android.material.bottomappbar.BottomAppBar':'Toolbar',
    'androidx.appcompat.widget.Toolbar':'Toolbar',
    'com.google.android.material.textfield.TextInputLayout':'LinearLayout',
    'androidx.appcompat.widget.AppCompatCheckBox':'CheckBox',
    'com.google.android.material.checkbox.MaterialCheckBox':'CheckBox',
    'androidx.appcompat.widget.SwitchCompat':'Switch',
    'com.google.android.material.switchmaterial.SwitchMaterial':'Switch',
    'com.google.android.material.materialswitch.MaterialSwitch':'Switch',
    'androidx.appcompat.widget.AppCompatRadioButton':'RadioButton',
    'com.google.android.material.radiobutton.MaterialRadioButton':'RadioButton',
    'androidx.appcompat.widget.AppCompatSpinner':'Spinner',
    'androidx.appcompat.widget.AppCompatSeekBar':'SeekBar',
    'androidx.appcompat.widget.AppCompatRatingBar':'RatingBar',
    'androidx.appcompat.widget.AppCompatToggleButton':'ToggleButton',
    'androidx.appcompat.widget.AppCompatAutoCompleteTextView':'AutoCompleteTextView',
    'androidx.appcompat.widget.AppCompatCheckedTextView':'CheckedTextView',
    'androidx.appcompat.widget.LinearLayoutCompat':'LinearLayout',
    'androidx.appcompat.widget.SearchView':'SearchView',
    'com.google.android.material.search.SearchBar':'SearchView',
    'com.google.android.material.search.SearchView':'FrameLayout',
    'androidx.appcompat.widget.ActionMenuView':'ActionMenuView',
    'androidx.appcompat.widget.ActionBarContainer':'FrameLayout',
    'androidx.appcompat.widget.ContentFrameLayout':'FrameLayout',
    'androidx.appcompat.widget.FitWindowsLinearLayout':'LinearLayout',
    'androidx.appcompat.widget.FitWindowsFrameLayout':'FrameLayout',
    'androidx.appcompat.widget.DialogTitle':'TextView',
    'androidx.appcompat.widget.AlertDialogLayout':'LinearLayout',
    'androidx.appcompat.widget.ButtonBarLayout':'LinearLayout',
    'androidx.core.widget.NestedScrollView':'ScrollView',
    'androidx.cardview.widget.CardView':'com.google.android.material.card.MaterialCardView',
    'com.google.android.material.imageview.ShapeableImageView':'ImageView',
    'com.google.android.material.chip.Chip':'Button',
    'com.google.android.material.chip.ChipGroup':'LinearLayout',
    'com.google.android.material.tabs.TabLayout':'TabWidget',
    'com.google.android.material.tabs.TabItem':'Button',
    'com.google.android.material.progressindicator.LinearProgressIndicator':'ProgressBar',
    'com.google.android.material.progressindicator.CircularProgressIndicator':'ProgressBar',
    'com.google.android.material.progressindicator.LinearWavyProgressIndicator':'ProgressBar',
    'com.google.android.material.progressindicator.CircularWavyProgressIndicator':'ProgressBar',
    'com.google.android.material.loadingindicator.LoadingIndicator':'ProgressBar',
    'com.google.android.material.bottomsheet.BottomSheetDragHandleView':'View',
    'com.google.android.material.snackbar.SnackbarContentLayout':'LinearLayout',
    'com.google.android.material.floatingtoolbar.FloatingToolbarLayout':'LinearLayout',
    'com.google.android.material.button.MaterialButtonGroup':'LinearLayout',
    'com.google.android.material.button.MaterialSplitButton':'LinearLayout',
    'com.google.android.material.carousel.MaskableFrameLayout':'FrameLayout',
    'com.google.android.material.slider.Slider':'SeekBar',
    'com.google.android.material.slider.RangeSlider':'SeekBar',
    'com.google.android.material.appbar.AppBarLayout':'LinearLayout',
    'com.google.android.material.appbar.CollapsingToolbarLayout':'FrameLayout',
    'com.google.android.material.bottomnavigation.BottomNavigationView':'LinearLayout',
    'com.google.android.material.navigation.NavigationBarView':'LinearLayout',
    'com.google.android.material.navigation.NavigationView':'LinearLayout',
    'com.google.android.material.navigationrail.NavigationRailView':'LinearLayout',
    'com.google.android.material.button.MaterialButtonToggleGroup':'LinearLayout',
    'androidx.coordinatorlayout.widget.CoordinatorLayout':'FrameLayout',
    'CoordinatorLayout':'FrameLayout',
    'androidx.drawerlayout.widget.DrawerLayout':'FrameLayout',
    'androidx.swiperefreshlayout.widget.SwipeRefreshLayout':'FrameLayout',
    'android.support.v4.widget.SwipeRefreshLayout':'FrameLayout',
    'androidx.viewpager2.widget.ViewPager2':'ViewFlipper',
    'androidx.viewpager.widget.ViewPager':'ViewFlipper',
    'android.support.v4.view.ViewPager':'ViewFlipper',
    'androidx.constraintlayout.widget.ConstraintLayout':'ConstraintLayout',
    'androidx.constraintlayout.widget.Guideline':'View',
    'androidx.constraintlayout.widget.Barrier':'View',
    'androidx.constraintlayout.widget.Group':'View',
    'androidx.constraintlayout.widget.Placeholder':'View',
    'android.support.constraint.Placeholder':'View',
    'androidx.constraintlayout.helper.widget.Flow':'View',
    'android.support.constraint.helper.Flow':'View',
    'androidx.constraintlayout.helper.widget.Layer':'View',
    'android.support.constraint.helper.Layer':'View',
    'androidx.preference.internal.PreferenceImageView':'ImageView',
    'androidx.gridlayout.widget.GridLayout':'GridLayout',
    'androidx.fragment.app.FragmentContainerView':'FrameLayout',
    'fragment':'FrameLayout',
    'ViewStub':'View',
    'android.view.View':'View',
    'android.view.ViewStub':'View',
}
TAGS={'GridLayout':'Grid','TwoLineListItem':'Relative','TableLayout':'Table','TableRow':'TableRow','LinearLayout':'Linear','FrameLayout':'Frame','MediaController':'MediaController','VideoView':'VideoView','RelativeLayout':'Relative','ConstraintLayout':'ConstraintLayout','AbsoluteLayout':'Absolute','ViewAnimator':'ViewAnimator','ViewSwitcher':'ViewSwitcher','TextSwitcher':'TextSwitcher','ImageSwitcher':'ImageSwitcher','ViewFlipper':'ViewFlipper','ScrollView':'Scroll','HorizontalScrollView':'HorizontalScroll','TextView':'Text','Chronometer':'Chronometer','TextClock':'TextClock','DigitalClock':'TextClock','AnalogClock':'AnalogClock','CheckedTextView':'CheckedText','CheckBox':'CheckBox','RadioButton':'RadioButton','RadioGroup':'RadioGroup','Switch':'Switch','SeekBar':'SeekBar','ProgressBar':'ProgressBar','RatingBar':'RatingBar','ToggleButton':'ToggleButton','Button':'Button','ImageButton':'Icon','ZoomButton':'Zoom','ZoomControls':'ZoomControls','ImageView':'Image','QuickContactBadge':'QuickContactBadge','EditText':'Edit','View':'View','Space':'Space'}
TAGS['com.google.android.material.divider.MaterialDivider']='MaterialDivider'
TAGS['com.google.android.material.card.MaterialCardView']='MaterialCard'
TAGS['Spinner']='Spinner'
TAGS['ListView']='ListView'
TAGS['GridView']='GridView'
TAGS['ExpandableListView']='ExpandableListView'
TAGS['SearchView']='SearchView'
TAGS['Toolbar']='Toolbar'
TAGS['ActionMenuView']='ActionMenuView'
TAGS['StackView']='StackView'
TAGS['Gallery']='Gallery'
TAGS['TabWidget']='TabWidget'
TAGS['TabHost']='TabHost'
TAGS['DialerFilter']='DialerFilter'
TAGS['SlidingDrawer']='SlidingDrawer'
TAGS['AdapterViewFlipper']='AdapterViewFlipper'
TAGS['TimePicker']='TimePicker'
TAGS['NumberPicker']='NumberPicker'
TAGS['CalendarView']='CalendarView'
TAGS['DatePicker']='DatePicker'
TAGS['AutoCompleteTextView']='AutoComplete'
TAGS['MultiAutoCompleteTextView']='MultiAutoComplete'
ATTRS_EXTRA_SEARCH={'iconifiedByDefault','queryHint'}
ATTRS_EXTRA_TOOLBAR={'title','subtitle','navigationIcon','navigationContentDescription','contentInsetStart','contentInsetEnd','contentInsetLeft','contentInsetRight','contentInsetStartWithNavigation','contentInsetEndWithActions','titleMargin','titleMarginStart','titleMarginEnd','titleMarginTop','titleMarginBottom','titleTextColor','subtitleTextColor','titleTextAppearance','subtitleTextAppearance'}
ATTRS_EXTRA_TIME={'timePickerMode'}
ATTRS_EXTRA_CALENDAR={'minDate','maxDate','firstDayOfWeek','datePickerMode','spinnersShown','calendarViewShown'}
ATTRS_EXTRA_AUTO={'completionThreshold','dropDownHeight'}
ATTRS_EXTRA_LIST={'choiceMode','dividerHeight','headerDividersEnabled','footerDividersEnabled'}
ATTRS_EXTRA_GRID={'numColumns','columnWidth','horizontalSpacing','verticalSpacing','stretchMode'}
ATTRS_EXTRA_SPINNER={'entries','spinnerMode','prompt','dropDownWidth','dropDownHorizontalOffset','dropDownVerticalOffset'}
ATTRS={'tabStripLeft','tabStripRight','tabStripEnabled','spacing','unselectedAlpha','animationDuration','handle','content','topOffset','bottomOffset','allowSingleTap','animateOnClick','loopViews','alignmentMode','useDefaultMargins','layout_columnWeight','layout_rowWeight','layout_rowSpan','layout_columnSpan','layout_row','rowCount','columnCount','baselineAligned','collapseColumns','stretchColumns','shrinkColumns','layout_column','layout_span','ignoreGravity','id','layout_width','layout_height','minWidth','minHeight','maxWidth','maxHeight','orientation','layout_weight','layout_x','layout_y','weightSum','layout_gravity','divider','showDividers','dividerPadding','layoutDirection','measureAllChildren','fillViewport','clipToPadding','clipChildren','autoStart','flipInterval','padding','paddingLeft','paddingTop','paddingRight','paddingBottom','paddingStart','paddingEnd','paddingHorizontal','paddingVertical','text','textOn','textOff','hint','contentDescription','textSize','letterSpacing','lineSpacingExtra','lineSpacingMultiplier','inputType','format','countDown','format12Hour','format24Hour','inAnimation','outAnimation','textStyle','textAllCaps','minLines','maxLines','lines','singleLine','ellipsize','textColor','background','foreground','foregroundGravity','foregroundInsidePadding','alpha','gravity','src','scaleType','adjustViewBounds','tint','tintMode','imageAlpha','enabled','clickable','checked','checkedButton','min','max','progress','secondaryProgress','indeterminate','numStars','rating','stepSize','isIndicator','mirrorForRtl','visibility'}
MARGINS={'layout_margin','layout_marginHorizontal','layout_marginVertical','layout_marginStart','layout_marginEnd','layout_marginLeft','layout_marginRight','layout_marginTop','layout_marginBottom'}
RELATIVE_ANCHORS={'layout_toLeftOf':0,'layout_toRightOf':1,'layout_above':2,'layout_below':3,'layout_alignLeft':4,'layout_alignRight':5,'layout_alignTop':6,'layout_alignBottom':7,'layout_toStartOf':8,'layout_toEndOf':9,'layout_alignStart':10,'layout_alignEnd':11,'layout_alignBaseline':22}
RELATIVE_FLAGS={'layout_alignParentLeft':12,'layout_alignParentRight':13,'layout_alignParentTop':14,'layout_alignParentBottom':15,'layout_alignParentStart':16,'layout_alignParentEnd':17,'layout_centerHorizontal':18,'layout_centerVertical':19,'layout_centerInParent':20,'layout_alignWithParentIfMissing':21}
ATTRS.update(MARGINS | set(RELATIVE_ANCHORS) | set(RELATIVE_FLAGS))
ATTRS.update({'progressTint','progressBackgroundTint','secondaryProgressTint','indeterminateTint','thumbTint','tickMark','tickMarkTint','thumbTintMode','tickMarkTintMode'})
ATTRS.update({'checkMarkTint','checkMarkGravity','checkMarkTintMode'})
ATTRS.update({'buttonTint','buttonTintMode'})
ATTRS.add('trackTint')
ATTRS.add('showText')
ATTRS.update({'switchMinWidth','switchPadding','thumbTextPadding'})
ATTRS.add('disabledAlpha')
ATTRS.add('splitTrack')
ATTRS.add('checkMark')
ATTRS.add('button')
ATTRS.add('thumb')
ATTRS.add('textAppearance')
ATTRS.update({'drawableLeft','drawableRight','drawableTop','drawableBottom','drawableStart','drawableEnd','drawablePadding'})
ATTRS.update({'drawableTint','drawableTintMode'})
ATTRS.update({'childDivider','groupIndicator','childIndicator','indicatorLeft','indicatorRight','indicatorStart','indicatorEnd','childIndicatorLeft','childIndicatorRight','childIndicatorStart','childIndicatorEnd'})
ATTRS.update(ATTRS_EXTRA_TOOLBAR)
ATTRS.add('baselineAlignedChildIndex')
ATTRS.update({'cropToPadding','baseline','baselineAlignBottom'})
ATTRS.update(ATTRS_EXTRA_SPINNER | ATTRS_EXTRA_LIST | ATTRS_EXTRA_GRID | ATTRS_EXTRA_AUTO | ATTRS_EXTRA_CALENDAR | ATTRS_EXTRA_TIME | ATTRS_EXTRA_SEARCH)
ATTRS.update({'backgroundTint','elevation','textAlignment','focusable','scrollbars'})
ATTRS.update({'fontFamily','textIsSelectable','autoLink','overScrollMode','imeOptions','fitsSystemWindows','longClickable','accessibilityHeading','animateLayoutChanges','theme'})
ATTRS.update({'importantForAccessibility','textColorHint','textColorLink','nestedScrollingEnabled','focusableInTouchMode','includeFontPadding','maxLength','typeface','linksClickable','scrollHorizontally','textDirection','scrollbarStyle','nextFocusLeft','nextFocusRight','nextFocusUp','nextFocusDown','nextFocusForward','transitionName','onClick','labelFor','descendantFocusability','stateListAnimator','translationZ'})
ATTRS.update({'clipToOutline','outlineProvider','saveEnabled','fadingEdge','fadeScrollbars','tooltipText','scaleX','scaleY','rotation','translationX','translationY','transformPivotX','transformPivotY','shadowColor','shadowRadius','shadowDx','shadowDy','textCursorDrawable','bufferType','cacheColorHint','drawSelectorOnTop','dropDownAnchor','layoutAnimation','progressTintMode','lineHeight','breakStrategy','hyphenationFrequency','justificationMode','autoSizeTextType','autoSizeMinTextSize','autoSizeMaxTextSize','autoSizeStepGranularity'})
def parse_locale_arg(raw_locale):
    if raw_locale is None:return (None,None,None)
    if not isinstance(raw_locale,str):raise ValueError('Invalid locale '+str(raw_locale))
    s=raw_locale.strip()
    if not s:return (None,None,None)
    if s.startswith('b+'):
        subtags=s.split('+')[1:]
        if not subtags or not re.fullmatch(r'[a-zA-Z]{2,3}',subtags[0]):raise ValueError('Invalid locale '+raw_locale)
        lang=LOCALE_LANG_CANON.get(subtags[0].lower(),subtags[0].lower())
        script=None;region=None;si=1
        if si<len(subtags) and re.fullmatch(r'[a-zA-Z]{4}',subtags[si]):script=subtags[si].title();si+=1
        if si<len(subtags) and re.fullmatch(r'[a-zA-Z]{2}|\d{3}',subtags[si]):region=subtags[si].upper();si+=1
        if si!=len(subtags):raise ValueError('Invalid locale '+raw_locale)
        return (lang,script,region)
    m=re.fullmatch(r'([a-zA-Z]{2,3})(?:[-_]([A-Z][a-z]{3}))?(?:[-_](?:r([A-Z]{2}|\d{3})|([a-zA-Z]{2}|\d{3})))?',s)
    if not m:raise ValueError('Invalid locale '+raw_locale)
    lang=LOCALE_LANG_CANON.get(m.group(1).lower(),m.group(1).lower())
    script=m.group(2).title() if m.group(2) else None
    reg=m.group(3) or m.group(4)
    region=reg.upper() if reg else None
    return (lang,script,region)
def compile_layout(path,resources=None,density=160,night=False,orientation='port',sw=0,merge_parent=None,custom_view_map=None,locale=None,ldrtl=None):
    if not 1 <= density <= 1000:raise ValueError("Invalid target density")
    if orientation not in {'port','land'}:raise ValueError("Invalid orientation "+str(orientation))
    if isinstance(sw,bool) or not isinstance(sw,int) or sw < 0:raise ValueError("Invalid smallest width "+str(sw))
    req_lang,req_script,req_region=parse_locale_arg(locale)
    if ldrtl is None:
        effective_ldrtl=(req_lang in RTL_LANGUAGES)
    elif isinstance(ldrtl,bool):
        effective_ldrtl=ldrtl
    else:
        raise ValueError("Invalid ldrtl "+str(ldrtl))
    resolved_custom_view_map={}
    if custom_view_map is not None:
        if isinstance(custom_view_map,(str,Path)):
            raw_map=json.loads(Path(custom_view_map).read_text())
        elif isinstance(custom_view_map,dict):
            raw_map=custom_view_map
        else:
            raise ValueError('Invalid custom view map')
        if not isinstance(raw_map,dict):raise ValueError('Custom view map must be a JSON object')
        for src_cls,dst_cls in raw_map.items():
            if not isinstance(src_cls,str) or not src_cls.strip() or not isinstance(dst_cls,str) or not dst_cls.strip():
                raise ValueError('Invalid custom view map entry')
            dst_clean=dst_cls.strip()
            dst_canon=TAG_ALIASES.get(dst_clean,dst_clean)
            if dst_canon.startswith('android.widget.') and dst_canon[len('android.widget.'):] in TAGS:
                dst_canon=dst_canon[len('android.widget.'):]
            if dst_canon not in TAGS and dst_clean not in {'androidx.recyclerview.widget.RecyclerView','android.support.v7.widget.RecyclerView','androidx.preference.PreferenceRecyclerView'}:
                raise ValueError('Unsupported custom view map target '+dst_clean)
            resolved_custom_view_map[src_cls.strip()]=dst_clean
    pool={}
    pool_root={}
    arrays={}
    styles={}
    fab_nodes=set()
    extended_fab_specs={}
    extended_fab_ext_specs={}
    bottom_app_bar_specs={}
    bottom_app_bar_ext_specs={}
    appbar_lift_specs={}
    appbar_ext_specs={}
    collapsing_toolbar_specs={}
    collapsing_toolbar_ext_specs={}
    recycler_layout_ext_specs={}
    carousel_layout_specs={}
    swipe_refresh_nodes=set()
    swipe_refresh_ext_specs={}
    drawer_layout_nodes=set()
    view_pager_specs={}
    view_pager_ext_specs={}
    range_slider_specs={}
    node_style_names={}
    drag_handle_nodes=set()
    loading_indicator_specs={}
    wavy_progress_specs={}
    alert_dialog_specs={}
    snackbar_nodes=set()
    tab_options_specs={}
    slider_options_specs={}
    text_input_ext_specs={}
    chip_group_specs={}
    floating_toolbar_nodes=set()
    button_group_specs={}
    split_button_nodes=set()
    carousel_item_nodes=set()
    chip_nodes=set()
    coordinator_nodes=set()
    text_input_specs={}
    button_variants={}
    chip_specs={}
    nav_bar_specs={}
    nav_item_specs={}
    nav_badge_specs={}
    nav_badge_ext_specs={}
    search_bar_specs={}
    search_view_ext_specs={}
    toggle_group_specs={}
    guideline_specs={}
    barrier_specs={}
    group_specs={}
    flow_specs={}
    flow_ext_specs={}
    layer_specs={}
    placeholder_specs={}
    def fmt_num(v):
        if v==0.0:return '0'
        if v.is_integer():return str(int(v))
        return str(round(v,6)).rstrip('0').rstrip('.')
    shapeable_image_nodes=set()
    track_corner_radii={}
    extra_strokes={}
    included_roots=set()
    include_override_attrs={}
    if resources is None:
        resource_roots=[]
    elif isinstance(resources,(str,Path)):
        resource_roots=[Path(resources)]
    else:
        resource_roots=[Path(r) for r in resources if r is not None]
    resource_root=resource_roots[0] if resource_roots else None
    def parse_res_dir_qualifiers(dir_name,base_type):
        if dir_name==base_type:
            return {'locale':0,'lddir':0,'sw':0,'w':0,'orient':0,'night':0,'density':None,'api':0}
        if not dir_name.startswith(base_type+'-'):return None
        tokens=dir_name[len(base_type)+1:].split('-')
        q={'locale':0,'lddir':0,'sw':0,'w':0,'orient':0,'night':0,'density':None,'api':0}
        seen=set()
        idx=0
        while idx<len(tokens):
            tok=tokens[idx]
            idx+=1
            if not tok:return None
            if tok in {'ldrtl','ldltr'}:
                if 'lddir' in seen:return None
                seen.add('lddir')
                if (tok=='ldrtl')!=bool(effective_ldrtl):return None
                q['lddir']=1
            elif tok in {'night','notnight'}:
                if 'night' in seen:return None
                seen.add('night')
                if (tok=='night')!=bool(night):return None
                q['night']=1
            elif tok in {'land','port'}:
                if 'orient' in seen:return None
                seen.add('orient')
                if tok!=orientation:return None
                q['orient']=1
            elif re.fullmatch(r'sw\d+dp',tok):
                if 'sw' in seen:return None
                seen.add('sw')
                val=int(tok[2:-2])
                if val<=0 or sw<val:return None
                q['sw']=val
            elif re.fullmatch(r'w\d+dp',tok):
                if 'w' in seen:return None
                seen.add('w')
                val=int(tok[1:-2])
                if val<=0 or sw<val:return None
                q['w']=val
            elif re.fullmatch(r'v\d+',tok):
                if 'api' in seen:return None
                seen.add('api')
                val=int(tok[1:])
                if val<=0:return None
                q['api']=val
            elif tok in DENSITY_QUALIFIERS:
                if 'density' in seen:return None
                seen.add('density')
                q['density']=tok
            elif re.fullmatch(r'[a-z]{2}',tok):
                if 'locale' in seen:return None
                seen.add('locale')
                dir_lang=LOCALE_LANG_CANON.get(tok,tok)
                dir_region=None
                if idx<len(tokens) and re.fullmatch(r'r(?:[A-Z]{2}|\d{3})',tokens[idx]):
                    dir_region=tokens[idx][1:].upper()
                    idx+=1
                if req_lang is None or dir_lang!=req_lang:return None
                if dir_region is not None:
                    if req_region!=dir_region:return None
                    q['locale']=3
                else:
                    q['locale']=1
            elif tok.startswith('b+'):
                if 'locale' in seen:return None
                seen.add('locale')
                subtags=tok.split('+')[1:]
                if not subtags or not re.fullmatch(r'[a-z]{2,3}',subtags[0]):return None
                dir_lang=LOCALE_LANG_CANON.get(subtags[0],subtags[0])
                dir_script=None;dir_region=None;si=1
                if si<len(subtags) and re.fullmatch(r'[A-Za-z]{4}',subtags[si]):dir_script=subtags[si].title();si+=1
                if si<len(subtags) and re.fullmatch(r'[A-Za-z]{2}|\d{3}',subtags[si]):dir_region=subtags[si].upper();si+=1
                if si!=len(subtags):return None
                if req_lang is None or dir_lang!=req_lang:return None
                if dir_script is not None and req_script!=dir_script:return None
                if dir_region is not None and req_region!=dir_region:return None
                if dir_script is not None and dir_region is not None:q['locale']=4
                elif dir_region is not None:q['locale']=3
                elif dir_script is not None:q['locale']=2
                else:q['locale']=1
            else:
                return None
        return q
    def qualifier_rank(q,for_drawable_xml=False):
        if for_drawable_xml:
            d=q['density']
            if d=='anydpi':d_rank=(3,0)
            elif d=='nodpi':d_rank=(1,0)
            elif d is not None:
                dpi=DENSITY_QUALIFIERS[d]
                d_rank=(2,-dpi if dpi>=density else -(10000-dpi))
            else:
                d_rank=(0,0)
        else:
            d_rank=(1 if q['density'] is not None else 0,0)
        return (q['locale'],q['lddir'],q['sw'],q['w'],q['orient'],q['night'],d_rank,q['api'])
    def matching_res_dirs(res_dir,base_type,for_drawable_xml=False):
        if not res_dir.is_dir():return []
        matched=[]
        for sub in sorted(res_dir.iterdir()):
            if not sub.is_dir():continue
            q=parse_res_dir_qualifiers(sub.name,base_type)
            if q is not None:
                matched.append((qualifier_rank(q,for_drawable_xml=for_drawable_xml),q,sub))
        matched.sort(key=lambda item:(item[0],item[2].name))
        return matched
    for root_idx,res_dir in enumerate(resource_roots):
        local_pool={}
        local_arrays={}
        local_styles={}
        for _,_,vdir in matching_res_dirs(res_dir,'values'):
            dir_arrays=set()
            dir_styles=set()
            for file in sorted(vdir.glob('*.xml')):
                for n in ET.parse(file).getroot():
                    if n.tag in {'string','color','dimen','bool','integer'}:
                        raw_val=''.join(n.itertext())
                        local_pool['@'+n.tag+'/'+n.attrib['name']]=raw_val if n.tag=='string' else raw_val.strip()
                    elif n.tag=='item' and n.attrib.get('type') in {'string','color','dimen','bool','integer'} and 'name' in n.attrib:
                        kind=n.attrib['type']; raw_val=''.join(n.itertext())
                        local_pool['@'+kind+'/'+n.attrib['name']]=raw_val if kind=='string' else raw_val.strip()
                    elif n.tag in {'string-array','integer-array','array'}:
                        name=n.attrib.get('name')
                        if not name or name in dir_arrays:raise ValueError('Invalid or duplicate '+n.tag)
                        if any(child.tag!='item' or len(child) for child in n):raise ValueError('Unsupported '+n.tag+' item')
                        dir_arrays.add(name)
                        local_arrays[name]=[''.join(child.itertext()) if n.tag=='string-array' else ''.join(child.itertext()).strip() for child in n]
                    elif n.tag=='style':
                        name=n.attrib.get('name')
                        if not name:raise ValueError('Style has no name')
                        if name in dir_styles:raise ValueError('Duplicate style '+name)
                        dir_styles.add(name)
                        local_styles[name]=n
        for k,v in local_pool.items():
            if k not in pool:
                pool[k]=v
                pool_root[k]=root_idx
        for name,vals in local_arrays.items():
            if name not in arrays:arrays[name]=vals
        for name,st in local_styles.items():
            if name not in styles:styles[name]=st
    def find_resource_file(subdir,filename):
        for res_dir in resource_roots:
            best=None
            for _,_,sdir in matching_res_dirs(res_dir,subdir):
                candidate=sdir/filename
                if candidate.is_file():best=candidate
            if best is not None:return best
        return None
    def find_color_xml(name):
        for root_idx,res_dir in enumerate(resource_roots):
            best=None
            for _,_,cdir in matching_res_dirs(res_dir,'color'):
                candidate=cdir/(name+'.xml')
                if candidate.is_file():best=candidate
            if best is not None:return root_idx,best
        return None,None
    def find_drawable_xml(name):
        for res_dir in resource_roots:
            best=None
            for _,_,ddir in matching_res_dirs(res_dir,'drawable',for_drawable_xml=True):
                candidate=ddir/(name+'.xml')
                if candidate.is_file():best=candidate
            if best is not None:return best
        return None
    def find_drawable_candidates(name):
        for res_dir in resource_roots:
            png_groups={}
            nodpi_groups={}
            best_xml=None
            best_xml_cfg=None
            for _,q,ddir in matching_res_dirs(res_dir,'drawable',for_drawable_xml=True):
                cfg_key=(q['locale'],q['lddir'],q['sw'],q['w'],q['orient'],q['night'],q['api'])
                for png_file in (ddir/(name+'.png'),ddir/(name+'.9.png')):
                    if png_file.is_file():
                        dq=q['density']
                        if dq=='nodpi':
                            if cfg_key in nodpi_groups and nodpi_groups[cfg_key].parent==ddir:
                                raise ValueError('Ambiguous PNG drawable '+name)
                            nodpi_groups[cfg_key]=png_file
                        elif dq!='anydpi':
                            dpi=160 if dq is None else DENSITY_QUALIFIERS[dq]
                            png_groups.setdefault(cfg_key,[]).append((dpi,png_file))
                xml_cand=ddir/(name+'.xml')
                if xml_cand.is_file():
                    best_xml=xml_cand
                    best_xml_cfg=cfg_key
            best_png_cfg=max(png_groups.keys() | nodpi_groups.keys(),default=None)
            if best_png_cfg is None and best_xml is None:continue
            if best_xml is not None and (best_png_cfg is None or best_xml_cfg>best_png_cfg):
                return [],None,best_xml
            if best_png_cfg is not None and (best_xml is None or best_png_cfg>best_xml_cfg):
                return png_groups.get(best_png_cfg,[]),nodpi_groups.get(best_png_cfg),None
            return png_groups.get(best_png_cfg,[]),nodpi_groups.get(best_png_cfg),best_xml
        return [],None,None
    def is_builtin_theme_name(name):
        return (
            name in {'Theme','AppTheme'}
            or name.startswith(('Theme.','Base.Theme.','ThemeOverlay.','Base.ThemeOverlay.','AppTheme.','Base.AppTheme.','Platform.Theme'))
            or name.endswith(('Theme','NoActionBar'))
            or '.Theme.' in name
        )
    def is_builtin_style_name(name, is_android=False):
        if is_android:
            return name.startswith(('Widget.','TextAppearance.','Theme.','ThemeOverlay.','Animation.')) or name in {'Widget','TextAppearance','Theme'}
        return name in BUILTIN_TEXT_APPEARANCES or name in BUILTIN_WIDGET_STYLES or name in BUILTIN_SHAPE_APPEARANCES or name in BUILTIN_STYLE_EXACT or name.startswith(BUILTIN_STYLE_PREFIXES)
    def style_parent_name(sname):
        st=styles[sname]
        if 'parent' in st.attrib:
            p=st.attrib['parent'].strip()
            if not p:return None
            for pref in ('@style/','@android:style/','android:'):
                if p.startswith(pref):return p[len(pref):]
            return p
        if '.' in sname:
            prefix=sname.rsplit('.',1)[0]
            if prefix in styles or is_builtin_theme_name(prefix) or is_builtin_style_name(prefix):return prefix
        return None
    def is_theme_style(sname, seen=()):
        if sname in seen:return False
        if is_builtin_theme_name(sname):return True
        if sname not in styles:return False
        parent=style_parent_name(sname)
        return is_theme_style(parent, seen+(sname,)) if parent else False
    def collect_theme_items(sname, seen=()):
        if sname in seen or sname not in styles:return {}
        parent=style_parent_name(sname)
        items=collect_theme_items(parent, seen+(sname,)) if parent and parent in styles else {}
        for item in styles[sname]:
            if item.tag=='item':
                k=item.attrib.get('name','').strip()
                if k:items[k]=''.join(item.itertext()).strip()
        return items
    theme_attrs={}
    theme_names=[sname for sname in styles if is_theme_style(sname)]
    if theme_names:
        resolved_themes={sname:collect_theme_items(sname) for sname in theme_names}
        theme_parents={style_parent_name(sname) for sname in theme_names if style_parent_name(sname) in styles}
        def is_dark_theme(sname, seen=()):
            if sname in seen:return False
            low=sname.lower()
            if 'light' in low and not ('night' in low and 'daynight' not in low):return False
            if ('dark' in low and 'darkactionbar' not in low) or ('night' in low and 'daynight' not in low):return True
            if sname in resolved_themes:
                tt=resolved_themes[sname].get('theme_type','').lower()
                if tt=='dark':return True
                if tt=='light':return False
            if sname in styles:
                p=style_parent_name(sname)
                if p:return is_dark_theme(p, seen+(sname,))
            return False
        def theme_rank(sname):
            low=sname.lower()
            is_overlay=any(tok in low for tok in ('overlay','dialog','alert','splash','switch','fullscreen','mediapreview','scribble'))
            is_dark=is_dark_theme(sname)
            is_base=low.startswith('base.')
            is_parent=sname in theme_parents
            is_light='light' in low
            if night:
                return (1 if is_overlay else 0, 0 if is_dark else 1, 1 if is_base else 0, 1 if is_parent else 0, 1 if is_light else 0, -len(resolved_themes[sname]), sname)
            return (1 if is_overlay else 0, 1 if is_dark else 0, 1 if is_base else 0, 1 if is_parent else 0, 0 if is_light else 1, -len(resolved_themes[sname]), sname)
        has_non_dark=any(not is_dark_theme(s) and not any(tok in s.lower() for tok in ('overlay','dialog','alert','splash','switch','fullscreen','mediapreview','scribble')) for s in theme_names)
        has_dark=any(is_dark_theme(s) and not any(tok in s.lower() for tok in ('overlay','dialog','alert','splash','switch','fullscreen','mediapreview','scribble')) for s in theme_names)
        for sname in sorted(theme_names, key=theme_rank):
            low=sname.lower()
            is_dark=is_dark_theme(sname)
            is_light='light' in low and not is_dark
            for k,v in resolved_themes[sname].items():
                if not night and has_non_dark and is_dark and (k.startswith('android:') or k in THEME_COLORS or k in THEME_DIMENS or k in THEME_STYLE_REFS):
                    continue
                if night and has_dark and is_light and not is_dark and (k.startswith('android:') or k in THEME_COLORS or k in THEME_DIMENS or k in THEME_STYLE_REFS):
                    continue
                theme_attrs.setdefault(k,v)
    def parse_theme_ref(v):
        if v.startswith('?android:attr/'):return ('android',v[len('?android:attr/'):])
        if v.startswith('?attr/'):return ('app',v[len('?attr/'):])
        if v.startswith('?android:') and '/' not in v and len(v)>len('?android:'):return ('android',v[len('?android:'):])
        if v.startswith('?') and '/' not in v and ':' not in v and len(v)>1:return ('app',v[1:])
        return None
    def lookup_theme_attr(v):
        parsed=parse_theme_ref(v)
        if parsed is None:return None
        ns,name=parsed
        if ns=='android':
            if 'android:'+name in theme_attrs:return theme_attrs['android:'+name]
            if name in theme_attrs:return theme_attrs[name]
        else:
            if name in theme_attrs:return theme_attrs[name]
            if 'android:'+name in theme_attrs:return theme_attrs['android:'+name]
        return None
    def canonical_view_tag(tag):
        if tag in resolved_custom_view_map:tag=resolved_custom_view_map[tag]
        if tag in TAG_ALIASES:return TAG_ALIASES[tag]
        if tag.startswith('android.widget.'):
            short=tag[len('android.widget.'):]
            if short in TAGS:return short
        return tag
    def resolve_shape_style_layers(reference, stack=()):
        if reference.startswith('?'):
            if reference in stack:raise ValueError('Cyclic style inheritance: '+' -> '.join(stack+(reference,)))
            tval=lookup_theme_attr(reference)
            if tval is not None:
                return resolve_shape_style_layers(tval,stack+(reference,))
            tok=theme_token(reference)
            if tok in THEME_STYLE_REFS:
                mapped=THEME_STYLE_REFS[tok]
                if isinstance(mapped,str):return resolve_shape_style_layers(mapped,stack+(reference,))
            raise ValueError('Unsupported style reference '+reference)
        is_android=False
        if reference.startswith('@style/'):name=reference[len('@style/'):]
        elif reference.startswith('@android:style/'):name=reference[len('@android:style/'):]; is_android=True
        elif reference.startswith('android:'):name=reference[len('android:'):]; is_android=True
        else:raise ValueError('Unsupported style reference '+reference)
        if name in stack:raise ValueError('Cyclic style inheritance: '+' -> '.join(stack+(name,)))
        if not is_android and name in styles:
            style=styles[name]
            if 'parent' in style.attrib:
                parent=style.attrib['parent'].strip()
            elif '.' in name:
                prefix=name.rsplit('.',1)[0]
                parent=prefix if (prefix in styles or is_builtin_style_name(prefix)) else ''
            else:
                parent=''
            layers=[]
            if parent:
                parent_ref=parent if parent.startswith(('@style/','@android:style/','android:','?')) else '@style/'+parent
                layers.extend(resolve_shape_style_layers(parent_ref,stack+(name,)))
            own={}
            for item in style:
                if item.tag!='item':raise ValueError('Unsupported shapeAppearance item in '+name)
                key=item.attrib.get('name','').removeprefix('app:').removeprefix('android:')
                val=''.join(item.itertext()).strip()
                if key not in SHAPE_APPEARANCE_KEYS:raise ValueError('Unsupported shapeAppearance item '+key+' in '+name)
                own[key]=val
            layers.append(own)
            return layers
        if name in BUILTIN_SHAPE_APPEARANCES:
            return [dict(BUILTIN_SHAPE_APPEARANCES[name])]
        if is_builtin_style_name(name, is_android):
            return []
        raise ValueError('Unresolved style '+reference)
    def resolve_style_chain(reference, stack=()):
        if not reference or reference in stack:return []
        if reference.startswith('?'):
            tval=lookup_theme_attr(reference)
            tok=theme_token(reference)
            head=[tok] if tok else []
            if tval is not None:return head+resolve_style_chain(tval,stack+(reference,))
            if tok in THEME_STYLE_REFS:
                mapped=THEME_STYLE_REFS[tok]
                if isinstance(mapped,str):return head+resolve_style_chain(mapped,stack+(reference,))
            return head
        is_android=False
        if reference.startswith('@style/'):name=reference[len('@style/'):]
        elif reference.startswith('@android:style/'):name=reference[len('@android:style/'):]; is_android=True
        elif reference.startswith('android:'):name=reference[len('android:'):]; is_android=True
        else:return []
        if name in stack:return []
        if not is_android and name in styles:
            st=styles[name]
            if 'parent' in st.attrib:parent=st.attrib['parent'].strip()
            elif '.' in name:
                prefix=name.rsplit('.',1)[0]
                parent=prefix if (prefix in styles or is_builtin_style_name(prefix)) else ''
            else:parent=''
            if parent:
                parent_ref=parent if parent.startswith(('@style/','@android:style/','android:','?')) else '@style/'+parent
                return [name]+resolve_style_chain(parent_ref,stack+(name,))
        return [name]
    def detect_button_variant(orig_tag, style_names):
        for sname in style_names:
            if 'OutlinedButton' in sname or sname.endswith('.Outlined') or sname in {'materialButtonOutlinedStyle','materialIconButtonOutlinedStyle'}:
                return 'outlined'
            if 'TonalButton' in sname or sname.endswith('.Tonal') or sname in {'materialButtonTonalStyle','materialIconButtonFilledTonalStyle'}:
                return 'tonal'
            if 'TextButton' in sname or 'Borderless' in sname or sname in {'materialButtonTextStyle','borderlessButtonStyle','materialIconButtonStyle'} or sname.endswith('.IconButton'):
                return 'text'
            if 'ElevatedButton' in sname or sname=='materialButtonElevatedStyle':
                return 'elevated'
        for sname in style_names:
            if 'UnelevatedButton' in sname or sname.endswith('.Filled') or sname in {'Widget.Material3.Button','Widget.MaterialComponents.Button','materialButtonStyle','materialIconButtonFilledStyle'}:
                return 'filled'
        if orig_tag=='com.google.android.material.button.MaterialButton':
            return 'filled'
        return None
    def style_attributes(reference,stack=(),target=None):
        if reference.startswith('?'):
            if reference in stack:raise ValueError('Cyclic style inheritance: '+' -> '.join(stack+(reference,)))
            tval=lookup_theme_attr(reference)
            if tval is not None:
                return style_attributes(tval,stack+(reference,),target)
            tok=theme_token(reference)
            if tok in THEME_STYLE_REFS:
                mapped=THEME_STYLE_REFS[tok]
                if isinstance(mapped,dict):return dict(mapped)
                return style_attributes(mapped,stack+(reference,),target)
            raise ValueError('Unsupported style reference '+reference)
        is_android=False
        if reference.startswith('@style/'):name=reference[len('@style/'):]
        elif reference.startswith('@android:style/'):name=reference[len('@android:style/'):]; is_android=True
        elif reference.startswith('android:'):name=reference[len('android:'):]; is_android=True
        else:raise ValueError('Unsupported style reference '+reference)
        if name in stack:raise ValueError('Cyclic style inheritance: '+' -> '.join(stack+(name,)))
        if not is_android and name in styles:
            style=styles[name]
            if 'parent' in style.attrib:
                parent=style.attrib['parent'].strip()
            elif '.' in name:
                prefix=name.rsplit('.',1)[0]
                parent=prefix if (prefix in styles or is_builtin_style_name(prefix)) else ''
            else:
                parent=''
            if parent:
                parent_ref=parent if parent.startswith(('@style/','@android:style/','android:','?')) else '@style/'+parent
                values=style_attributes(parent_ref,stack+(name,),target)
            else:
                values={}
            for item in style:
                if item.tag!='item':
                    raise ValueError('Unsupported style item in '+name)
                item_name=item.attrib.get('name','')
                item_val=''.join(item.itertext()).strip()
                if item_name.startswith('android:'):
                    key=item_name[len('android:'):]
                    if key in ATTRS:values[A+key]=item_val
                    elif key in STYLE_IGNORED_ANDROID_ATTRS:continue
                    else:raise ValueError('Unsupported style attribute '+key+' in '+name)
                else:
                    key=item_name.removeprefix('app:')
                    if key in MATERIAL_DIVIDER_ATTRS | MATERIAL_CARD_ATTRS | APP_COMMON_ATTRS | set(CONSTRAINT_ANCHORS) | set(CONSTRAINT_BIASES) | ATTRS_EXTRA_TOOLBAR | TOOLBAR_APP_EXTRA_ATTRS | APP_BUTTON_ATTRS | TEXT_INPUT_ATTRS | CHIP_ATTRS | CHIP_GROUP_ATTRS | TAB_LAYOUT_ATTRS | NAV_STYLE_ATTRS | TOGGLE_GROUP_ATTRS | EXTENDED_FAB_ATTRS | BOTTOM_APP_BAR_ATTRS | APP_BAR_ATTRS | COLLAPSING_TOOLBAR_ATTRS | SLIDER_ATTRS | LOADING_INDICATOR_ATTRS | PROGRESS_INDICATOR_ATTRS | ALERT_DIALOG_ATTRS | SNACKBAR_ATTRS | FLOATING_TOOLBAR_ATTRS | BUTTON_GROUP_ATTRS | FLOW_ATTRS | RECYCLER_VIEW_ATTRS | SEARCH_VIEW_ATTRS | SWIPE_REFRESH_ATTRS | CAROUSEL_ATTRS | VIEW_PAGER_ATTRS:
                        values[APP+key]=item_val
                    elif key in STYLE_UNPREFIXED_TO_ANDROID:
                        values[A+key]=item_val
                    elif key in STYLE_IGNORED_APP_ATTRS:
                        continue
                    else:raise ValueError('Unsupported style item in '+name)
            return values
        if name in {'Widget.Material3.MaterialDivider','Widget.Material3.MaterialDivider.Heavy'}:
            if target!='com.google.android.material.divider.MaterialDivider':raise ValueError('MaterialDivider style on unsupported view')
            return {APP+'dividerThickness':'8dp'} if name.endswith('.Heavy') else {}
        if name in {'Widget.Material3.CardView.Outlined','Widget.Material3.CardView.Filled','Widget.Material3.CardView.Elevated'}:
            if target not in {'com.google.android.material.card.MaterialCardView','androidx.cardview.widget.CardView'}:raise ValueError('MaterialCardView style on unsupported view')
            return {APP+'bhooCardVariant':name.rsplit('.',1)[-1]}
        if name in BUILTIN_TEXT_APPEARANCES:
            return dict(BUILTIN_TEXT_APPEARANCES[name])
        if name in BUILTIN_WIDGET_STYLES:
            vals=dict(BUILTIN_WIDGET_STYLES[name])
            if A+'orientation' in vals and canonical_view_tag(target or '')!='ProgressBar':
                vals.pop(A+'orientation',None)
            return vals
        if is_builtin_style_name(name, is_android):
            return {}
        raise ValueError('Unresolved style '+reference)
    def parse_text_style(raw, err_prefix='Unsupported textStyle '):
        parts=[p.strip() for p in raw.split('|')]
        if not parts or any(p not in {'normal','bold','italic'} for p in parts):
            raise ValueError(err_prefix+raw)
        return 'bold' in parts
    def match_material_icon_name(raw_icon):
        if not raw_icon or raw_icon=='@null':return ''
        val=raw_icon.strip()
        if val.startswith('?'):
            tval=lookup_theme_attr(val)
            if tval is not None:val=resolve(tval).strip()
            else:return ''
        else:
            val=resolve(val).strip()
        if val=='@null':return ''
        m=re.fullmatch(r'@(?:android:)?drawable/([a-zA-Z0-9_]+)',val)
        if not m:return ''
        stem=m.group(1).lower()
        def lookup(s):
            if s in MATERIAL_ICON_NAMES:return s
            if s in NAV_ICON_ALIASES:return NAV_ICON_ALIASES[s]
            return None
        hit=lookup(stem)
        if hit:return hit
        if stem.startswith('ic_'):
            stem=stem[3:]
            hit=lookup(stem)
            if hit:return hit
        suffixes=('_24dp','_18dp','_20dp','_36dp','_48dp','_24','_18','_20','_36','_48','_black','_white','_grey','_gray','_tint','_outline','_outlined','_filled','_rounded','_round','_sharp','_baseline','_border','_selector','_icon','_ic')
        changed=True
        while changed:
            changed=False
            for suf in suffixes:
                if stem.endswith(suf) and len(stem)>len(suf):
                    stem=stem[:-len(suf)]
                    hit=lookup(stem)
                    if hit:return hit
                    changed=True
                    break
        prefixes=('baseline_','outline_','outlined_','round_','rounded_','sharp_','filled_','twotone_','mtrl_','abc_','nav_','menu_','tab_','action_','vector_')
        changed=True
        while changed:
            changed=False
            for pref in prefixes:
                if stem.startswith(pref) and len(stem)>len(pref):
                    stem=stem[len(pref):]
                    hit=lookup(stem)
                    if hit:return hit
                    changed=True
                    break
        return ''
    def parse_badge_ext_attrs(attrib, base_attrib=None):
        merged={}
        if base_attrib:
            for k in BADGE_EXT_ATTRS:
                if APP+k in base_attrib:merged[APP+k]=base_attrib[APP+k]
        for k in BADGE_EXT_ATTRS:
            if APP+k in attrib:merged[APP+k]=attrib[APP+k]
        if not merged:return None
        grav='top_end'
        if APP+'badgeGravity' in merged:
            raw_g=resolve(merged[APP+'badgeGravity']).strip()
            grav={'top_end':'top_end','top_start':'top_start','bottom_end':'bottom_end','bottom_start':'bottom_start',
                  'TOP_END':'top_end','TOP_START':'top_start','BOTTOM_END':'bottom_end','BOTTOM_START':'bottom_start'}.get(raw_g)
            if grav is None:raise ValueError('Unsupported badgeGravity '+raw_g)
        max_c='4'
        if APP+'badgeMaxCharacterCount' in merged:
            raw_m=resolve(merged[APP+'badgeMaxCharacterCount']).strip()
            if not re.fullmatch(r'\d+',raw_m) or int(raw_m)<=0:
                raise ValueError('Invalid badgeMaxCharacterCount '+raw_m)
            max_c=str(int(raw_m))
        hoff='0'
        if APP+'badgeHorizontalOffset' in merged:
            raw_h=merged[APP+'badgeHorizontalOffset']
            res_h=resolve(raw_h)
            try: hoff=fmt_num(float(margin_number(res_h)))
            except ValueError: raise ValueError('Invalid badgeHorizontalOffset '+raw_h)
        voff='0'
        if APP+'badgeVerticalOffset' in merged:
            raw_v=merged[APP+'badgeVerticalOffset']
            res_v=resolve(raw_v)
            try: voff=fmt_num(float(margin_number(res_v)))
            except ValueError: raise ValueError('Invalid badgeVerticalOffset '+raw_v)
        return f'{grav}|{max_c}|{hoff},{voff}'
    def parse_nav_menu(menu_ref, default_badge_attrs=None):
        menu_name=menu_ref.split('/',1)[1]
        menu_file=find_resource_file('menu',menu_name+'.xml')
        if menu_file is None:return [],[],[]
        mroot=ET.parse(menu_file).getroot()
        if mroot.tag!='menu':raise ValueError('Expected a menu root in '+menu_ref)
        collected=[]
        badges=[]
        def parse_order(raw, default_val):
            if raw is None:return default_val
            val=resolve(raw).strip()
            if not re.fullmatch(r'-?(?:0[xX][0-9a-fA-F]+|[0-9]+)',val):raise ValueError('Invalid menu integer '+val)
            return int(val,16 if val.lstrip('-').lower().startswith('0x') else 10)&0xffff
        def collect(container, group_visible=True, group_order=0):
            for child in container:
                if child.tag=='group':
                    g_vis=group_visible
                    if A+'visible' in child.attrib:
                        gv=resolve(child.attrib[A+'visible']).strip()
                        if gv not in {'true','false'}:raise ValueError('Invalid menu boolean '+gv)
                        g_vis=(gv=='true')
                    g_ord=parse_order(child.attrib.get(A+'orderInCategory'),group_order)
                    collect(child,g_vis,g_ord)
                elif child.tag=='item':
                    i_vis=group_visible
                    if A+'visible' in child.attrib:
                        iv=resolve(child.attrib[A+'visible']).strip()
                        if iv not in {'true','false'}:raise ValueError('Invalid menu boolean '+iv)
                        i_vis=(iv=='true')
                    if not i_vis:continue
                    submenus=[c for c in child if c.tag=='menu']
                    if submenus:
                        for sm in submenus:collect(sm,True,0)
                        continue
                    raw_id=child.attrib.get(A+'id','').strip()
                    if raw_id:
                        norm_id=id_key(raw_id)
                        if norm_id is None:raise ValueError('Invalid menu item id '+raw_id)
                    else:
                        norm_id=''
                    raw_title=child.attrib.get(A+'title',child.attrib.get(A+'titleCondensed',''))
                    title=resolve(raw_title) if raw_title else ''
                    title=title.replace('|','/').replace('\n',' ')
                    icon_name=match_material_icon_name(child.attrib.get(A+'icon',''))
                    i_ord=parse_order(child.attrib.get(A+'orderInCategory'),group_order)
                    idx=len(collected)
                    badge_ext=parse_badge_ext_attrs(child.attrib,default_badge_attrs)
                    collected.append((i_ord,idx,f'{norm_id}|{title}|{icon_name}',badge_ext))
                    if APP+'badgeText' in child.attrib:
                        raw_badge=child.attrib[APP+'badgeText']
                        badge_text=resolve(raw_badge) if raw_badge else ''
                        if badge_text!='@null' and norm_id:
                            badge_text=badge_text.replace('\n',' ').strip()
                            badges.append((i_ord,idx,f'{norm_id}:{badge_text}'))
                else:
                    raise ValueError('Unsupported menu element '+child.tag)
        collect(mroot)
        collected.sort(key=lambda item:(item[0],item[1]))
        badges.sort(key=lambda item:(item[0],item[1]))
        badge_exts=[f'{final_idx}|{item[3]}' for final_idx,item in enumerate(collected) if item[3] is not None]
        return [item[2] for item in collected],[item[2] for item in badges],badge_exts
    def unwrap_databinding(n):
        if n.tag!='layout':return n
        views=[c for c in n if c.tag!='data']
        if len(views)!=1:raise ValueError('DataBinding <layout> must contain exactly one root view')
        return views[0]
    def expand(n,stack=(),in_merge=False):
        if n.tag=='layout':
            return expand(unwrap_databinding(n),stack,in_merge)
        if n.tag=='include':
            if not resource_roots:raise ValueError('include needs --resources')
            ref=n.attrib.get('layout','')
            match=re.fullmatch(r'@layout/([a-z][a-z0-9_]*)',ref)
            if not match:raise ValueError('Invalid include layout '+ref)
            name=match.group(1)
            if name in stack:raise ValueError('Cyclic include: '+' -> '.join(stack+(name,)))
            include_android_allowed={A+'id',A+'layout_width',A+'layout_height',A+'visibility',A+'layout_gravity',A+'layout_weight'} | {A+k for k in MARGINS | set(RELATIVE_ANCHORS) | set(RELATIVE_FLAGS)}
            include_app_allowed={APP+k for k in (set(CONSTRAINT_ANCHORS) | set(CONSTRAINT_BIASES) | APP_LAYOUT_PARAMS_ATTRS)}
            for key in n.attrib:
                if key!='layout' and not key.startswith(TOOLS) and key not in include_android_allowed and key not in include_app_allowed:
                    raise ValueError('Unsupported include attribute '+key)
            override={k:v for k,v in n.attrib.items() if k.startswith(A) or k.startswith(APP)}
            if any(k.startswith(A+'layout_') for k in override) and not {A+'layout_width',A+'layout_height'} <= override.keys():
                raise ValueError('include layout overrides require both width and height')
            file=find_resource_file('layout',f'{name}.xml')
            if file is None:raise ValueError('Unresolved layout '+ref)
            child=unwrap_databinding(ET.parse(file).getroot())
            if child.tag=='merge':
                if override:raise ValueError('include of merge cannot override attributes')
                expanded_children=expand(child,stack+(name,),True)
                for ec in expanded_children:included_roots.add(ec)
                return expanded_children
            override_android={k[len(A):] for k in override if k.startswith(A)}
            child.attrib.update(override)
            expanded_children=expand(child,stack+(name,))
            for ec in expanded_children:
                included_roots.add(ec)
                include_override_attrs.setdefault(ec,set()).update(override_android)
            return expanded_children
        if n.tag=='merge':
            if not in_merge:raise ValueError('merge must be included into a parent layout')
            if any(not k.startswith(TOOLS) for k in n.attrib):raise ValueError('Unsupported merge attribute')
            result=[]
            for child in n:result.extend(expand(child,stack))
            return result
        if n.tag=='requestFocus':
            if len(n) or any(not k.startswith(TOOLS) for k in n.attrib):raise ValueError('Unsupported requestFocus attribute or child')
            return []
        if n.tag in resolved_custom_view_map:
            n.tag=resolved_custom_view_map[n.tag]
        orig_tag=n.tag
        leading_children=[]
        style_names=resolve_style_chain(n.attrib['style']) if 'style' in n.attrib else []
        if 'style' in n.attrib:
            reference=n.attrib.pop('style')
            horizontal_styles={'?android:attr/progressBarStyleHorizontal','?attr/progressBarStyleHorizontal','@android:style/Widget.ProgressBar.Horizontal','@android:style/Widget.Material.ProgressBar.Horizontal'}
            if canonical_view_tag(n.tag)=='ProgressBar' and reference in horizontal_styles:
                n.attrib[A+'orientation']='horizontal'
            else:
                if n.tag=='com.google.android.material.divider.MaterialDivider':
                    reference={'?attr/materialDividerHeavyStyle':'@style/Widget.Material3.MaterialDivider.Heavy','?attr/materialDividerStyle':'@style/Widget.Material3.MaterialDivider'}.get(reference,reference)
                if n.tag in {'com.google.android.material.card.MaterialCardView','androidx.cardview.widget.CardView'}:
                    reference={'?attr/materialCardViewStyle':'@style/Widget.Material3.CardView.Outlined','?attr/materialCardViewOutlinedStyle':'@style/Widget.Material3.CardView.Outlined','?attr/materialCardViewFilledStyle':'@style/Widget.Material3.CardView.Filled','?attr/materialCardViewElevatedStyle':'@style/Widget.Material3.CardView.Elevated'}.get(reference,reference)
                attrs=style_attributes(reference,target=n.tag)
                for dual_k in ('lineHeight','fontFamily','autoSizeTextType','autoSizeMinTextSize','autoSizeMaxTextSize','autoSizeStepGranularity'):
                    if APP+dual_k in n.attrib and A+dual_k not in n.attrib:attrs.pop(A+dual_k,None)
                    elif A+dual_k in n.attrib and APP+dual_k not in n.attrib:attrs.pop(APP+dual_k,None)
                attrs.update(n.attrib);n.attrib.clear();n.attrib.update(attrs)
        if A+'textAppearance' in n.attrib:
            canonical_tag=canonical_view_tag(n.tag)
            if canonical_tag not in {'TextView','CheckedTextView','Chronometer','TextClock','Button','ToggleButton','CheckBox','RadioButton','Switch','EditText','AutoCompleteTextView','MultiAutoCompleteTextView'}:raise ValueError('textAppearance unsupported on '+n.tag)
            appearance=n.attrib.pop(A+'textAppearance')
            attrs=style_attributes(appearance,target=n.tag)
            allowed={A+'textSize',A+'textStyle',A+'textColor',A+'textAllCaps',A+'letterSpacing',A+'fontFamily',APP+'fontFamily',A+'typeface',A+'textColorHint',A+'textColorLink',A+'lineSpacingMultiplier',A+'lineHeight',APP+'lineHeight',A+'breakStrategy',A+'hyphenationFrequency',A+'justificationMode',A+'includeFontPadding'}
            if set(attrs)-allowed:raise ValueError('Unsupported textAppearance attribute')
            for dual_k in ('lineHeight','fontFamily'):
                if APP+dual_k in n.attrib and A+dual_k not in n.attrib:attrs.pop(A+dual_k,None)
                elif A+dual_k in n.attrib and APP+dual_k not in n.attrib:attrs.pop(APP+dual_k,None)
            attrs.update(n.attrib);n.attrib.clear();n.attrib.update(attrs)
        for key in list(n.attrib):
            if key.startswith(TOOLS):del n.attrib[key]
        if style_names:
            node_style_names[n]=style_names
        if orig_tag in {'androidx.coordinatorlayout.widget.CoordinatorLayout','CoordinatorLayout'}:
            coordinator_nodes.add(n)
        if orig_tag=='androidx.appcompat.widget.AlertDialogLayout' or any('MaterialAlertDialog' in s for s in style_names):
            dlg_icon=''
            if A+'icon' in n.attrib:
                dlg_icon=match_material_icon_name(n.attrib.pop(A+'icon'))
            elif APP+'icon' in n.attrib and canonical_view_tag(n.tag)!='Button':
                dlg_icon=match_material_icon_name(n.attrib.pop(APP+'icon'))
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in (ALERT_DIALOG_ATTRS-{'icon'}):
                    number(n.attrib[k])
                    del n.attrib[k]
            alert_dialog_specs[n]=dlg_icon
        if orig_tag=='com.google.android.material.bottomsheet.BottomSheetDragHandleView':
            drag_handle_nodes.add(n)
            for tk in (A+'tint',APP+'tint'):
                if tk in n.attrib:
                    raw_t=resolve(n.attrib.pop(tk)).strip()
                    if raw_t!='@null':
                        static_color_rgba(raw_t)
                        if A+'backgroundTint' not in n.attrib and APP+'backgroundTint' not in n.attrib:
                            n.attrib[A+'backgroundTint']=raw_t
        if orig_tag=='com.google.android.material.loadingindicator.LoadingIndicator':
            li_contained='1' if any('Contained' in s for s in style_names) else '0'
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in LOADING_INDICATOR_ATTRS:
                    aname=k[len(APP):]
                    if aname=='containerColor':
                        raw_cc=resolve(n.attrib[k]).strip()
                        if raw_cc!='@null':
                            cc_rgba=static_color_rgba(raw_cc)
                            li_contained='1' if cc_rgba[3]>0 else '0'
                    elif aname=='indicatorColor':
                        raw_ic=resolve(n.attrib[k]).strip()
                        if raw_ic!='@null' and not raw_ic.startswith('@array/'):
                            static_color_rgba(raw_ic)
                            if A+'progressTint' not in n.attrib:
                                n.attrib[A+'progressTint']=n.attrib[k]
                    elif aname in {'indicatorSize','containerWidth','containerHeight'}:
                        number(n.attrib[k])
                    elif aname in {'showDelay','minHideDelay'}:
                        raw_d=resolve(n.attrib[k]).strip()
                        if not re.fullmatch(r'\d+',raw_d):raise ValueError('Invalid '+aname+' '+raw_d)
                    del n.attrib[k]
            loading_indicator_specs[n]=li_contained
        if orig_tag=='com.google.android.material.snackbar.SnackbarContentLayout':
            snackbar_nodes.add(n)
            if A+'orientation' not in n.attrib:n.attrib[A+'orientation']='horizontal'
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in SNACKBAR_ATTRS:
                    aname=k[len(APP):]
                    if aname=='animationMode':
                        am=resolve(n.attrib[k]).strip()
                        if am not in {'slide','fade'}:raise ValueError('Unsupported snackbar animationMode '+am)
                    elif aname=='maxActionInlineWidth':
                        number(n.attrib[k])
                    del n.attrib[k]
        if orig_tag=='com.google.android.material.floatingtoolbar.FloatingToolbarLayout':
            floating_toolbar_nodes.add(n)
            if A+'orientation' not in n.attrib:n.attrib[A+'orientation']='horizontal'
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in FLOATING_TOOLBAR_ATTRS:
                    bv=resolve(n.attrib[k]).strip()
                    if bv not in {'true','false'}:raise ValueError('Invalid '+k[len(APP):]+' '+bv)
                    del n.attrib[k]
        if orig_tag=='com.google.android.material.button.MaterialButtonGroup':
            if A+'orientation' not in n.attrib:n.attrib[A+'orientation']='horizontal'
            if A+'spacing' in n.attrib:
                bg_spacing=number(n.attrib.pop(A+'spacing'))
                n.attrib.pop(APP+'spacing',None)
            elif APP+'spacing' in n.attrib:
                bg_spacing=number(n.attrib.pop(APP+'spacing'))
            elif any('Connected' in s for s in style_names):
                bg_spacing='2.0'
            else:
                bg_spacing='12.0'
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in BUTTON_GROUP_ATTRS:
                    aname=k[len(APP):]
                    if aname=='overflowMode':
                        om=resolve(n.attrib[k]).strip()
                        if om not in {'none','menu','wrap'}:raise ValueError('Unsupported overflowMode '+om)
                    del n.attrib[k]
            button_group_specs[n]=bg_spacing
        if orig_tag=='com.google.android.material.button.MaterialSplitButton':
            split_button_nodes.add(n)
            if A+'orientation' not in n.attrib:n.attrib[A+'orientation']='horizontal'
            if A+'spacing' in n.attrib:number(n.attrib.pop(A+'spacing'))
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in BUTTON_GROUP_ATTRS:
                    aname=k[len(APP):]
                    if aname=='spacing':number(n.attrib[k])
                    elif aname=='overflowMode':
                        om=resolve(n.attrib[k]).strip()
                        if om not in {'none','menu','wrap'}:raise ValueError('Unsupported overflowMode '+om)
                    del n.attrib[k]
        if orig_tag=='com.google.android.material.carousel.MaskableFrameLayout':
            carousel_item_nodes.add(n)
        if orig_tag in {'Button','android.widget.Button','com.google.android.material.button.MaterialButton','androidx.appcompat.widget.AppCompatButton'}:
            btn_var=detect_button_variant(orig_tag,style_names)
            if btn_var is not None:button_variants[n]=btn_var
        if n.tag=='com.google.android.material.textfield.TextInputLayout':
            if A+'hint' in n.attrib:
                for child in n:
                    if canonical_view_tag(child.tag)=='EditText' and A+'hint' not in child.attrib:
                        child.attrib[A+'hint']=n.attrib[A+'hint']
            if APP+'boxBackgroundMode' in n.attrib:
                til_mode=resolve(n.attrib[APP+'boxBackgroundMode']).strip()
                if til_mode not in {'outline','filled','none'}:raise ValueError('Unsupported TextInputLayout boxBackgroundMode '+til_mode)
            elif any('OutlinedBox' in s or s=='textInputOutlinedStyle' for s in style_names):
                til_mode='outline'
            elif any('FilledBox' in s or s=='textInputFilledStyle' for s in style_names):
                til_mode='filled'
            else:
                til_mode='filled'
            til_corner='4'
            for c_attr in ('boxCornerRadiusTopStart','boxCornerRadiusTopEnd','boxCornerRadiusBottomStart','boxCornerRadiusBottomEnd'):
                if APP+c_attr in n.attrib:
                    til_corner=number(n.attrib[APP+c_attr])
                    break
            til_stroke=''
            if APP+'boxStrokeColor' in n.attrib:
                raw_sc=resolve(n.attrib[APP+'boxStrokeColor'])
                if raw_sc!='@null':til_stroke=rgba_to_csv(static_color_rgba(raw_sc))
            hint_enabled='true'
            if APP+'hintEnabled' in n.attrib:
                hint_enabled=resolve(n.attrib[APP+'hintEnabled'])
                if hint_enabled not in {'true','false'}:raise ValueError('Invalid hintEnabled '+hint_enabled)
            til_hint=''
            if hint_enabled!='false':
                if A+'hint' in n.attrib:
                    til_hint=resolve(n.attrib[A+'hint'])
                else:
                    for child in n.iter():
                        if child is n:continue
                        if canonical_view_tag(child.tag) in {'EditText','AutoCompleteTextView','MultiAutoCompleteTextView'} and A+'hint' in child.attrib:
                            til_hint=resolve(child.attrib[A+'hint'])
                            break
            til_hint=til_hint.replace('|','/').replace('\n',' ')
            helper_enabled='true'
            if APP+'helperTextEnabled' in n.attrib:
                helper_enabled=resolve(n.attrib[APP+'helperTextEnabled'])
                if helper_enabled not in {'true','false'}:raise ValueError('Invalid helperTextEnabled '+helper_enabled)
            til_helper=''
            if helper_enabled!='false' and APP+'helperText' in n.attrib:
                til_helper=resolve(n.attrib[APP+'helperText']).replace('|','/').replace('\n',' ')
            til_err='0'
            if APP+'errorEnabled' in n.attrib:
                ev=resolve(n.attrib[APP+'errorEnabled'])
                if ev not in {'true','false'}:raise ValueError('Invalid errorEnabled '+ev)
                til_err='1' if ev=='true' else '0'
            til_end_icon='none'
            if APP+'endIconMode' in n.attrib:
                eim=resolve(n.attrib[APP+'endIconMode']).strip()
                if eim in {'none','clear_text','password_toggle','dropdown_menu'}:til_end_icon=eim
                elif eim=='custom':til_end_icon='none'
                else:raise ValueError('Unsupported endIconMode '+eim)
            elif APP+'passwordToggleEnabled' in n.attrib:
                pt=resolve(n.attrib[APP+'passwordToggleEnabled'])
                if pt not in {'true','false'}:raise ValueError('Invalid passwordToggleEnabled '+pt)
                til_end_icon='password_toggle' if pt=='true' else 'none'
            elif any('ExposedDropdownMenu' in s for s in style_names):
                til_end_icon='dropdown_menu'
            text_input_specs[n]=f'{til_mode}|{til_corner}|{til_stroke}|{til_hint}|{til_helper}|{til_err}|{til_end_icon}'
            if any(APP+ek in n.attrib for ek in ('counterEnabled','counterMaxLength','prefixText','suffixText','startIconDrawable')):
                til_counter_en='0'
                if APP+'counterEnabled' in n.attrib:
                    cev=resolve(n.attrib[APP+'counterEnabled']).strip()
                    if cev not in {'true','false'}:raise ValueError('Invalid counterEnabled '+cev)
                    til_counter_en='1' if cev=='true' else '0'
                til_counter_max='0'
                if APP+'counterMaxLength' in n.attrib:
                    cmv=resolve(n.attrib[APP+'counterMaxLength']).strip()
                    if not re.fullmatch(r'-1|\d+',cmv):raise ValueError('Invalid counterMaxLength '+cmv)
                    til_counter_max=str(max(0,int(cmv)))
                til_prefix=''
                if APP+'prefixText' in n.attrib:
                    pv=resolve(n.attrib[APP+'prefixText'])
                    if pv!='@null':til_prefix=pv.replace('|','/').replace('\n',' ')
                til_suffix=''
                if APP+'suffixText' in n.attrib:
                    sv=resolve(n.attrib[APP+'suffixText'])
                    if sv!='@null':til_suffix=sv.replace('|','/').replace('\n',' ')
                til_start_icon=''
                if APP+'startIconDrawable' in n.attrib:
                    raw_si=resolve(n.attrib[APP+'startIconDrawable']).strip()
                    if raw_si and raw_si!='@null' and not raw_si.startswith('?') and not re.fullmatch(r'@(?:android:)?drawable/[a-zA-Z0-9_]+',raw_si):
                        raise ValueError('Invalid startIconDrawable '+raw_si)
                    til_start_icon=match_material_icon_name(n.attrib[APP+'startIconDrawable'])
                text_input_ext_specs[n]=f'{til_counter_en}|{til_counter_max}|{til_prefix}|{til_suffix}|{til_start_icon}'
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in TEXT_INPUT_ATTRS:
                    del n.attrib[k]
        if n.tag=='com.google.android.material.floatingactionbutton.FloatingActionButton':
            fab_nodes.add(n)
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in {'fabSize','fabCustomSize','borderWidth','elevation','maxImageSize','hoveredFocusedTranslationZ','pressedTranslationZ','rippleColor','useCompatPadding','ensureMinTouchTargetSize','showMotionSpec','hideMotionSpec'}:
                    if k==APP+'elevation' and A+'elevation' not in n.attrib:n.attrib[A+'elevation']=n.attrib[k]
                    elif k==APP+'fabSize':
                        fs=resolve(n.attrib[k]).strip()
                        if fs not in {'auto','normal','mini'}:raise ValueError('Unsupported fabSize '+fs)
                    elif k in {APP+'maxImageSize',APP+'fabCustomSize',APP+'borderWidth'}:
                        number(n.attrib[k])
                    del n.attrib[k]
        if n.tag=='com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton':
            efab_extended='1'
            efab_collapsed_size=56.0
            efab_strategy='auto'
            has_explicit_extended=APP+'extended' in n.attrib
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in EXTENDED_FAB_ATTRS:
                    aname=k[len(APP):]
                    if aname=='elevation' and A+'elevation' not in n.attrib:
                        n.attrib[A+'elevation']=n.attrib[k]
                    elif aname=='extended':
                        aval=resolve(n.attrib[k]).strip()
                        if aval not in {'true','false'}:raise ValueError('Invalid extended '+aval)
                        efab_extended='1' if aval=='true' else '0'
                    elif aname=='extendMotionSpec' and not has_explicit_extended:
                        raw_ms=n.attrib[k].strip()
                        if not raw_ms.startswith(('@animator/','@android:animator/','@anim/','@android:anim/')):
                            if resolve(raw_ms).strip()=='false':efab_extended='0'
                    elif aname=='collapsedSize':
                        aval=resolve(n.attrib[k]).strip()
                        if aval!='@null':efab_collapsed_size=float(number(n.attrib[k]))
                    elif aname=='extendStrategy':
                        aval=resolve(n.attrib[k]).strip()
                        if aval not in {'auto','wrap_content','match_parent'}:raise ValueError('Unsupported extendStrategy '+aval)
                        efab_strategy=aval
                    elif aname=='ensureMinTouchTargetSize':
                        aval=resolve(n.attrib[k]).strip()
                        if aval not in {'true','false'}:raise ValueError('Invalid ensureMinTouchTargetSize '+aval)
                    del n.attrib[k]
            extended_fab_specs[n]=efab_extended
            if efab_extended!='1' or efab_collapsed_size!=56.0 or efab_strategy!='auto':
                extended_fab_ext_specs[n]=f'{efab_extended}|{fmt_num(efab_collapsed_size)}|{efab_strategy}'
        if n.tag=='com.google.android.material.imageview.ShapeableImageView':
            shapeable_image_nodes.add(n)
            s_color=n.attrib.pop(APP+'strokeColor',None)
            s_width=n.attrib.pop(APP+'strokeWidth',None)
            if s_color is not None or s_width is not None:
                extra_strokes[n]=(s_color,s_width)
            for cp_k,p_k in (('contentPadding','padding'),('contentPaddingLeft','paddingLeft'),('contentPaddingTop','paddingTop'),('contentPaddingRight','paddingRight'),('contentPaddingBottom','paddingBottom'),('contentPaddingStart','paddingStart'),('contentPaddingEnd','paddingEnd')):
                if APP+cp_k in n.attrib:
                    val=n.attrib.pop(APP+cp_k)
                    if A+p_k not in n.attrib:n.attrib[A+p_k]=val
        if n.tag=='com.google.android.material.chip.Chip':
            chip_nodes.add(n)
            chip_kind='assist'
            for sname in style_names:
                if 'Input' in sname or 'Entry' in sname:chip_kind='input';break
                if 'Filter' in sname or 'Choice' in sname:chip_kind='filter';break
                if 'Suggestion' in sname:chip_kind='suggestion';break
                if 'Assist' in sname or 'Action' in sname:chip_kind='assist';break
            close_vis='1' if chip_kind=='input' else '0'
            if APP+'closeIconVisible' in n.attrib:
                cv=resolve(n.attrib[APP+'closeIconVisible'])
                if cv not in {'true','false'}:raise ValueError('Invalid closeIconVisible '+cv)
                close_vis='1' if cv=='true' else '0'
            chip_icon_visible=True
            if APP+'chipIconVisible' in n.attrib:
                civ=resolve(n.attrib[APP+'chipIconVisible'])
                if civ not in {'true','false'}:raise ValueError('Invalid chipIconVisible '+civ)
                chip_icon_visible=(civ=='true')
            if APP+'checkedIconVisible' in n.attrib:
                chv=resolve(n.attrib[APP+'checkedIconVisible'])
                if chv not in {'true','false'}:raise ValueError('Invalid checkedIconVisible '+chv)
            for ck_key in (APP+'checkable',A+'checkable'):
                if ck_key in n.attrib:
                    ckv=resolve(n.attrib[ck_key])
                    if ckv not in {'true','false'}:raise ValueError('Invalid checkable '+ckv)
            n.attrib.pop(A+'checkable',None)
            chip_specs[n]=f'{chip_kind}|{close_vis}'
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in CHIP_ATTRS:
                    if k==APP+'chipBackgroundColor' and A+'backgroundTint' not in n.attrib:n.attrib[A+'backgroundTint']=n.attrib[k]
                    elif k==APP+'chipStrokeColor' and APP+'strokeColor' not in n.attrib:n.attrib[APP+'strokeColor']=n.attrib[k]
                    elif k==APP+'chipStrokeWidth' and APP+'strokeWidth' not in n.attrib:n.attrib[APP+'strokeWidth']=n.attrib[k]
                    elif k==APP+'chipCornerRadius' and APP+'cornerRadius' not in n.attrib:n.attrib[APP+'cornerRadius']=n.attrib[k]
                    elif k==APP+'chipIcon' and chip_icon_visible and A+'drawableStart' not in n.attrib:n.attrib[A+'drawableStart']=n.attrib[k]
                    elif k==APP+'chipIconTint' and A+'drawableTint' not in n.attrib and resolve(n.attrib[k])!='@null':n.attrib[A+'drawableTint']=n.attrib[k]
                    elif k==APP+'chipMinHeight' and A+'minHeight' not in n.attrib:n.attrib[A+'minHeight']=n.attrib[k]
                    del n.attrib[k]
        if n.tag=='com.google.android.material.chip.ChipGroup':
            if A+'orientation' not in n.attrib:n.attrib[A+'orientation']='horizontal'
            cg_single_line='0'
            cg_single_sel='0'
            cg_base_space=None
            cg_h_space=None
            cg_v_space=None
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in CHIP_GROUP_ATTRS:
                    aname=k[len(APP):]
                    if aname=='singleLine':
                        bv=resolve(n.attrib[k]).strip()
                        if bv not in {'true','false'}:raise ValueError('Invalid singleLine '+bv)
                        cg_single_line='1' if bv=='true' else '0'
                    elif aname=='singleSelection':
                        bv=resolve(n.attrib[k]).strip()
                        if bv not in {'true','false'}:raise ValueError('Invalid singleSelection '+bv)
                        cg_single_sel='1' if bv=='true' else '0'
                    elif aname=='selectionRequired':
                        bv=resolve(n.attrib[k]).strip()
                        if bv not in {'true','false'}:raise ValueError('Invalid selectionRequired '+bv)
                    elif aname=='chipSpacing':
                        cg_base_space=number(n.attrib[k])
                    elif aname=='chipSpacingHorizontal':
                        cg_h_space=number(n.attrib[k])
                    elif aname=='chipSpacingVertical':
                        cg_v_space=number(n.attrib[k])
                    elif aname=='checkedChip':
                        raw_cc=resolve(n.attrib[k]).strip()
                        if raw_cc and raw_cc!='@null' and id_key(raw_cc) is None:
                            raise ValueError('Invalid checkedChip '+raw_cc)
                    del n.attrib[k]
            cg_h=cg_h_space if cg_h_space is not None else (cg_base_space if cg_base_space is not None else '8.0')
            cg_v=cg_v_space if cg_v_space is not None else (cg_base_space if cg_base_space is not None else '8.0')
            chip_group_specs[n]=f'{cg_single_line}|{cg_single_sel}|{cg_h}|{cg_v}'
        if n.tag in {'com.google.android.material.bottomnavigation.BottomNavigationView','com.google.android.material.navigation.NavigationBarView'}:
            if A+'orientation' not in n.attrib:n.attrib[A+'orientation']='horizontal'
            nav_label_mode='auto'
            nav_menu_ref=None
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in {'menu','itemIconTint','itemTextColor','labelVisibilityMode','itemActiveIndicatorStyle','itemBackground','itemIconSize','itemPaddingTop','itemPaddingBottom','itemTextAppearanceActive','itemTextAppearanceInactive','itemRippleColor','itemHorizontalTranslationEnabled'}:
                    if k==APP+'labelVisibilityMode':
                        lvm=resolve(n.attrib[k]).strip()
                        if lvm not in {'auto','selected','labeled','unlabeled'}:raise ValueError('Unsupported labelVisibilityMode '+lvm)
                        nav_label_mode=lvm
                    elif k==APP+'menu':
                        mv=resolve(n.attrib[k]).strip()
                        if mv!='@null' and not re.fullmatch(r'@menu/[a-z][a-z0-9_]*',mv):raise ValueError('Invalid menu reference '+mv)
                        if mv!='@null':nav_menu_ref=mv
                    del n.attrib[k]
            direct_badge_attrs={APP+bk:n.attrib.pop(APP+bk) for bk in BADGE_EXT_ATTRS if APP+bk in n.attrib}
            direct_badge_spec=parse_badge_ext_attrs(direct_badge_attrs)
            nav_bar_specs[n]=f'bar|{nav_label_mode}'
            if nav_menu_ref is not None:
                parsed_nav,parsed_badges,parsed_badge_exts=parse_nav_menu(nav_menu_ref,direct_badge_attrs or None)
                if parsed_nav:nav_item_specs[n]=parsed_nav
                if parsed_badges:nav_badge_specs[n]=parsed_badges
                if parsed_badge_exts:nav_badge_ext_specs[n]=parsed_badge_exts
                elif direct_badge_spec is not None:nav_badge_ext_specs[n]=[f'0|{direct_badge_spec}']
            elif direct_badge_spec is not None:
                nav_badge_ext_specs[n]=[f'0|{direct_badge_spec}']
        if n.tag=='com.google.android.material.navigationrail.NavigationRailView':
            nav_label_mode='auto'
            nav_menu_ref=None
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in {'menu','headerLayout','menuGravity','itemMinHeight','itemIconTint','itemTextColor','labelVisibilityMode','itemActiveIndicatorStyle','itemBackground','itemIconSize','itemPaddingTop','itemPaddingBottom','itemTextAppearanceActive','itemTextAppearanceInactive','itemRippleColor','paddingTopSystemWindowInsets','paddingBottomSystemWindowInsets'}:
                    if k==APP+'labelVisibilityMode':
                        lvm=resolve(n.attrib[k]).strip()
                        if lvm not in {'auto','selected','labeled','unlabeled'}:raise ValueError('Unsupported labelVisibilityMode '+lvm)
                        nav_label_mode=lvm
                    elif k==APP+'menuGravity':
                        mg=resolve(n.attrib[k]).strip()
                        if mg not in {'top','center','bottom'}:raise ValueError('Unsupported menuGravity '+mg)
                    elif k==APP+'menu':
                        mv=resolve(n.attrib[k]).strip()
                        if mv!='@null' and not re.fullmatch(r'@menu/[a-z][a-z0-9_]*',mv):raise ValueError('Invalid menu reference '+mv)
                        if mv!='@null':nav_menu_ref=mv
                    elif k==APP+'headerLayout':
                        hv=resolve(n.attrib[k]).strip()
                        if hv!='@null' and not re.fullmatch(r'@layout/[a-z][a-z0-9_]*',hv):raise ValueError('Invalid headerLayout '+hv)
                    del n.attrib[k]
            direct_badge_attrs={APP+bk:n.attrib.pop(APP+bk) for bk in BADGE_EXT_ATTRS if APP+bk in n.attrib}
            direct_badge_spec=parse_badge_ext_attrs(direct_badge_attrs)
            nav_bar_specs[n]=f'rail|{nav_label_mode}'
            if nav_menu_ref is not None:
                parsed_nav,parsed_badges,parsed_badge_exts=parse_nav_menu(nav_menu_ref,direct_badge_attrs or None)
                if parsed_nav:nav_item_specs[n]=parsed_nav
                if parsed_badges:nav_badge_specs[n]=parsed_badges
                if parsed_badge_exts:nav_badge_ext_specs[n]=parsed_badge_exts
                elif direct_badge_spec is not None:nav_badge_ext_specs[n]=[f'0|{direct_badge_spec}']
            elif direct_badge_spec is not None:
                nav_badge_ext_specs[n]=[f'0|{direct_badge_spec}']
        if n.tag=='com.google.android.material.navigation.NavigationView':
            nav_menu_ref=None
            nav_header_ref=None
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in {'menu','headerLayout','itemIconTint','itemTextColor','itemBackground','itemTextAppearance','itemShapeFillColor','itemMaxLines','itemIconSize','itemIconPadding','itemHorizontalPadding','itemVerticalPadding','itemShapeAppearance','itemShapeAppearanceOverlay','subheaderInsetStart','subheaderInsetEnd','dividerInsetStart','dividerInsetEnd','topInsetScrimEnabled','bottomInsetScrimEnabled','drawerLayoutCornerSize'}:
                    if k==APP+'headerLayout':
                        hv=resolve(n.attrib[k]).strip()
                        if hv!='@null' and not re.fullmatch(r'@layout/[a-z][a-z0-9_]*',hv):raise ValueError('Invalid headerLayout '+hv)
                        if hv!='@null':nav_header_ref=hv
                    elif k==APP+'menu':
                        mv=resolve(n.attrib[k]).strip()
                        if mv!='@null' and not re.fullmatch(r'@menu/[a-z][a-z0-9_]*',mv):raise ValueError('Invalid menu reference '+mv)
                        if mv!='@null':nav_menu_ref=mv
                    del n.attrib[k]
            direct_badge_attrs={APP+bk:n.attrib.pop(APP+bk) for bk in BADGE_EXT_ATTRS if APP+bk in n.attrib}
            direct_badge_spec=parse_badge_ext_attrs(direct_badge_attrs)
            nav_bar_specs[n]='drawer|labeled'
            if nav_menu_ref is not None:
                parsed_nav,parsed_badges,parsed_badge_exts=parse_nav_menu(nav_menu_ref,direct_badge_attrs or None)
                if parsed_nav:nav_item_specs[n]=parsed_nav
                if parsed_badges:nav_badge_specs[n]=parsed_badges
                if parsed_badge_exts:nav_badge_ext_specs[n]=parsed_badge_exts
                elif direct_badge_spec is not None:nav_badge_ext_specs[n]=[f'0|{direct_badge_spec}']
            elif direct_badge_spec is not None:
                nav_badge_ext_specs[n]=[f'0|{direct_badge_spec}']
            if nav_header_ref is not None and resource_roots:
                header_name=nav_header_ref.split('/',1)[1]
                if header_name in stack:raise ValueError('Cyclic include: '+' -> '.join(stack+(header_name,)))
                header_file=find_resource_file('layout',f'{header_name}.xml')
                if header_file is not None:
                    header_el=unwrap_databinding(ET.parse(header_file).getroot())
                    if header_el.tag=='merge':
                        header_expanded=expand(header_el,stack+(header_name,),True)
                    else:
                        header_expanded=expand(header_el,stack+(header_name,))
                    for ec in header_expanded:included_roots.add(ec)
                    leading_children.extend(header_expanded)
        if n.tag=='com.google.android.material.bottomappbar.BottomAppBar':
            bab_fab_align='center'
            bab_fab_anchor='cradle'
            bab_cradle_margin=5.0
            bab_cradle_radius=8.0
            bab_cradle_voff=0.0
            bab_hide_on_scroll='0'
            bab_has_ext=False
            bab_menu_ref=None
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in BOTTOM_APP_BAR_ATTRS:
                    aname=k[len(APP):]; aval=resolve(n.attrib[k]).strip()
                    if aname=='fabAlignmentMode':
                        if aval not in {'center','end'}:raise ValueError('Unsupported fabAlignmentMode '+aval)
                        bab_fab_align=aval
                        bab_has_ext=True
                    elif aname=='menu':
                        if aval!='@null' and not re.fullmatch(r'@menu/[a-z][a-z0-9_]*',aval):raise ValueError('Invalid menu reference '+aval)
                        if aval!='@null':bab_menu_ref=aval
                    elif aname=='fabAnimationMode' and aval not in {'scale','slide'}:raise ValueError('Unsupported fabAnimationMode '+aval)
                    elif aname=='fabAnchorMode':
                        if aval not in {'embed','cradle'}:raise ValueError('Unsupported fabAnchorMode '+aval)
                        bab_fab_anchor=aval
                        bab_has_ext=True
                    elif aname=='menuAlignmentMode' and aval not in {'auto','start'}:raise ValueError('Unsupported menuAlignmentMode '+aval)
                    elif aname in {'fabCradleMargin','fabCradleRoundedCornerRadius','fabCradleVerticalOffset','fabAlignmentModeEndMargin'}:
                        nv=float(number(n.attrib[k]))
                        if aname=='fabCradleMargin':bab_cradle_margin=nv;bab_has_ext=True
                        elif aname=='fabCradleRoundedCornerRadius':bab_cradle_radius=nv;bab_has_ext=True
                        elif aname=='fabCradleVerticalOffset':bab_cradle_voff=nv;bab_has_ext=True
                    elif aname in {'hideOnScroll','removeEmbeddedFabElevation','addElevationShadow','paddingBottomSystemWindowInsets','paddingLeftSystemWindowInsets','paddingRightSystemWindowInsets'}:
                        if aval not in {'true','false'}:raise ValueError('Invalid '+aname+' '+aval)
                        if aname=='hideOnScroll':bab_hide_on_scroll='1' if aval=='true' else '0';bab_has_ext=True
                    del n.attrib[k]
            direct_badge_attrs={APP+bk:n.attrib.pop(APP+bk) for bk in BADGE_EXT_ATTRS if APP+bk in n.attrib}
            direct_badge_spec=parse_badge_ext_attrs(direct_badge_attrs)
            bottom_app_bar_specs[n]=bab_fab_align
            if bab_has_ext:
                bottom_app_bar_ext_specs[n]=f'{bab_fab_align}|{bab_fab_anchor}|{fmt_num(bab_cradle_margin)},{fmt_num(bab_cradle_radius)},{fmt_num(bab_cradle_voff)}|{bab_hide_on_scroll}'
            if bab_menu_ref is not None:
                parsed_nav,parsed_badges,parsed_badge_exts=parse_nav_menu(bab_menu_ref,direct_badge_attrs or None)
                if parsed_nav:nav_item_specs[n]=parsed_nav
                if parsed_badges:nav_badge_specs[n]=parsed_badges
                if parsed_badge_exts:nav_badge_ext_specs[n]=parsed_badge_exts
                elif direct_badge_spec is not None:nav_badge_ext_specs[n]=[f'0|{direct_badge_spec}']
            elif direct_badge_spec is not None:
                nav_badge_ext_specs[n]=[f'0|{direct_badge_spec}']
        if n.tag=='com.google.android.material.search.SearchView':
            app_hint=n.attrib.pop(APP+'hint',None)
            sv_anchor=''
            if APP+'layout_anchor' in n.attrib:
                raw_anc=resolve(n.attrib.pop(APP+'layout_anchor')).strip()
                if raw_anc and raw_anc!='@null':
                    sv_anchor=id_key(raw_anc)
                    if sv_anchor is None:
                        raise ValueError('Invalid layout_anchor '+raw_anc)
            raw_sv_hint=n.attrib.get(A+'hint',app_hint if app_hint is not None else '')
            resolved_sv_hint=resolve(raw_sv_hint) if raw_sv_hint else ''
            sv_hint='' if resolved_sv_hint=='@null' else resolved_sv_hint.replace('|','/').replace('\r\n',' ').replace('\r',' ').replace('\n',' ')
            sv_hide_nav='0'
            sv_use_drawer='0'
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in SEARCH_VIEW_ATTRS:
                    aname=k[len(APP):]
                    aval=resolve(n.attrib[k]).strip()
                    if aname in {'hideNavigationIcon','useDrawerArrowDrawable','autoShowKeyboard','animateNavigationIcon','animateMenuItems','backHandlingEnabled'}:
                        if aval not in {'true','false'}:
                            raise ValueError('Invalid '+aname+' '+aval)
                        if aname=='hideNavigationIcon':sv_hide_nav='1' if aval=='true' else '0'
                        elif aname=='useDrawerArrowDrawable':sv_use_drawer='1' if aval=='true' else '0'
                    elif aname=='headerLayout':
                        if aval!='@null' and not re.fullmatch(r'@layout/[a-z][a-z0-9_]*',aval):
                            raise ValueError('Invalid headerLayout '+aval)
                    del n.attrib[k]
            search_view_ext_specs[n]=f'{sv_anchor}|{sv_hint}|{sv_hide_nav}|{sv_use_drawer}'
        if n.tag in {'com.google.android.material.search.SearchBar','androidx.appcompat.widget.SearchView'}:
            is_m3_search=n.tag=='com.google.android.material.search.SearchBar'
            app_hint=n.attrib.pop(APP+'hint',None)
            if is_m3_search:
                if A+'iconifiedByDefault' not in n.attrib and APP+'iconifiedByDefault' not in n.attrib:
                    n.attrib[A+'iconifiedByDefault']='false'
                if A+'hint' in n.attrib and A+'queryHint' not in n.attrib and APP+'queryHint' not in n.attrib:
                    n.attrib[A+'queryHint']=n.attrib[A+'hint']
                elif app_hint is not None and A+'queryHint' not in n.attrib and APP+'queryHint' not in n.attrib:
                    n.attrib[A+'queryHint']=app_hint
            if APP+'iconifiedByDefault' in n.attrib:
                val=n.attrib.pop(APP+'iconifiedByDefault')
                if A+'iconifiedByDefault' not in n.attrib:n.attrib[A+'iconifiedByDefault']=val
            for qk in ('queryHint','defaultQueryHint'):
                if APP+qk in n.attrib:
                    val=n.attrib.pop(APP+qk)
                    if A+'queryHint' not in n.attrib:n.attrib[A+'queryHint']=val
            if n.tag=='com.google.android.material.search.SearchBar':
                raw_sb_hint=n.attrib.get(A+'hint',app_hint if app_hint is not None else n.attrib.get(A+'queryHint',''))
                resolved_sb_hint=resolve(raw_sb_hint) if raw_sb_hint else ''
                search_bar_specs[n]='' if resolved_sb_hint=='@null' else resolved_sb_hint
                s_color=n.attrib.pop(APP+'strokeColor',None)
                s_width=n.attrib.pop(APP+'strokeWidth',None)
                if s_color is not None or s_width is not None:
                    extra_strokes[n]=(s_color,s_width)
                for k in list(n.attrib):
                    if k.startswith(APP) and k[len(APP):] in {'navigationIcon','navigationIconTint','menu','defaultMarginsEnabled','defaultScrollFlagsEnabled','forceDefaultNavigationOnClickListener','hideNavigationIcon','tintNavigationIcon','liftOnScrollColor','liftOnScroll','textCentered','adaptiveMaxWidthEnabled','startSiblingViewId','endSiblingViewId'}:
                        del n.attrib[k]
            else:
                for k in list(n.attrib):
                    if k.startswith(APP) and k[len(APP):] in {'searchIcon','closeIcon','goIcon','voiceIcon','commitIcon','searchHintIcon','queryBackground','submitBackground','showDividers','layout','suggestionRowLayout'}:
                        del n.attrib[k]
        if n.tag=='com.google.android.material.button.MaterialButtonToggleGroup':
            if A+'orientation' not in n.attrib:n.attrib[A+'orientation']='horizontal'
            if A+'spacing' in n.attrib:number(n.attrib.pop(A+'spacing'))
            tg_single='0'
            tg_required='0'
            tg_checked=''
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in ({'singleSelection','selectionRequired','checkedButton'} | BUTTON_GROUP_ATTRS):
                    if k==APP+'singleSelection':
                        bv=resolve(n.attrib[k]).strip()
                        if bv not in {'true','false'}:raise ValueError('Invalid singleSelection '+bv)
                        tg_single='1' if bv=='true' else '0'
                    elif k==APP+'selectionRequired':
                        bv=resolve(n.attrib[k]).strip()
                        if bv not in {'true','false'}:raise ValueError('Invalid selectionRequired '+bv)
                        tg_required='1' if bv=='true' else '0'
                    elif k==APP+'checkedButton':
                        raw_cb=resolve(n.attrib[k]).strip()
                        if raw_cb and raw_cb!='@null':
                            cb_id=id_key(raw_cb)
                            if cb_id is None:raise ValueError('Invalid checkedButton '+raw_cb)
                            tg_checked=cb_id
                    elif k==APP+'spacing':
                        number(n.attrib[k])
                    elif k==APP+'overflowMode':
                        om=resolve(n.attrib[k]).strip()
                        if om not in {'none','menu','wrap'}:raise ValueError('Unsupported overflowMode '+om)
                    del n.attrib[k]
            toggle_group_specs[n]=f'{tg_single}|{tg_required}|{tg_checked}'
        if n.tag=='com.google.android.material.tabs.TabLayout':
            tab_mode='fixed'
            tab_style='secondary' if any('secondary' in s.lower() for s in style_names) else 'primary'
            tab_indicator=''
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in TAB_LAYOUT_ATTRS:
                    aname=k[len(APP):]
                    if aname=='tabBackground' and A+'background' not in n.attrib:n.attrib[A+'background']=n.attrib[k]
                    elif aname=='tabMode':
                        tm=resolve(n.attrib[k]).strip()
                        if tm not in {'fixed','scrollable','auto'}:raise ValueError('Unsupported tabMode '+tm)
                        tab_mode='scrollable' if tm=='scrollable' else 'fixed'
                    elif aname=='tabGravity':
                        tg=resolve(n.attrib[k]).strip()
                        if tg not in {'fill','center','start'}:raise ValueError('Unsupported tabGravity '+tg)
                    elif aname in {'tabIndicatorColor','tabTextColor','tabSelectedTextColor'}:
                        cv=resolve(n.attrib[k]).strip()
                        if cv!='@null':
                            rgba=static_color_rgba(cv)
                            if aname=='tabIndicatorColor':tab_indicator=rgba_to_csv(rgba)
                    del n.attrib[k]
            tab_options_specs[n]=f'{tab_mode}|{tab_style}|{tab_indicator}'
        if n.tag=='com.google.android.material.tabs.TabItem':
            if A+'icon' in n.attrib:
                icon_ref=n.attrib.pop(A+'icon')
                if A+'drawableStart' not in n.attrib:n.attrib[A+'drawableStart']=icon_ref
            n.attrib.pop(A+'layout',None)
        if n.tag=='com.google.android.material.appbar.AppBarLayout':
            appbar_lifted=False
            appbar_lift_on_scroll=False
            appbar_lift_target=''
            appbar_lift_color=''
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in APP_BAR_ATTRS:
                    aname=k[len(APP):]; aval=resolve(n.attrib[k]).strip()
                    if aname in {'expanded','liftOnScroll','lifted'} and aval not in {'true','false'}:
                        raise ValueError('Invalid '+aname+' '+aval)
                    elif aname=='liftOnScrollTargetViewId':
                        if aval!='@null':
                            tid=id_key(aval)
                            if tid is None:raise ValueError('Invalid liftOnScrollTargetViewId '+aval)
                            appbar_lift_target=tid
                    elif aname=='liftOnScrollColor':
                        if aval!='@null':
                            appbar_lift_color=rgba_to_csv(static_color_rgba(aval))
                    elif aname=='statusBarForeground' and aval!='@null':
                        if aval.startswith('@drawable/'):
                            if not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',aval):raise ValueError('Invalid '+aname+' '+aval)
                        else:
                            static_color_rgba(aval)
                    if aname=='lifted' and aval=='true':
                        appbar_lifted=True
                    elif aname=='liftOnScroll' and aval=='true':
                        appbar_lift_on_scroll=True
                    del n.attrib[k]
            if appbar_lifted:
                raw_el=n.attrib.get(A+'elevation',n.attrib.get(APP+'elevation'))
                if raw_el is None or float(number(raw_el))<=0.0:
                    n.attrib[A+'elevation']='4dp'
                    n.attrib.pop(APP+'elevation',None)
            if appbar_lift_on_scroll:
                appbar_lift_specs[n]=f'1|{appbar_lift_target}' if appbar_lift_target else '1'
            if appbar_lift_color:
                appbar_ext_specs[n]=f'lift_color={appbar_lift_color}'
        if n.tag=='com.google.android.material.appbar.CollapsingToolbarLayout':
            ctl_title=''
            ctl_scrim=''
            ctl_collapse_mode='scale'
            ctl_max_lines=1
            ctl_scrim_dur=0
            ctl_subtitle=''
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in COLLAPSING_TOOLBAR_ATTRS:
                    aname=k[len(APP):]
                    if aname=='titlePositionInterpolator':
                        del n.attrib[k]; continue
                    aval=resolve(n.attrib[k]).strip()
                    if aname=='title':
                        if aval!='@null':
                            ctl_title=aval.replace('|','/').replace('\n',' ')
                    elif aname=='subtitle':
                        if aval!='@null':
                            ctl_subtitle=aval.replace('|','/').replace('\n',' ')
                    elif aname in {'expandedTitleMargin','expandedTitleMarginStart','expandedTitleMarginEnd','expandedTitleMarginTop','expandedTitleMarginBottom','expandedTitleSpacing','scrimVisibleHeightTrigger'}:
                        number(n.attrib[k])
                    elif aname=='scrimAnimationDuration':
                        if not re.fullmatch(r'\d+',aval):raise ValueError('Invalid scrimAnimationDuration '+aval)
                        ctl_scrim_dur=int(aval)
                    elif aname in {'titleEnabled','forceApplySystemWindowInsetTop','extraMultilineHeightEnabled'}:
                        if aval not in {'true','false'}:raise ValueError('Invalid '+aname+' '+aval)
                    elif aname=='titleCollapseMode':
                        if aval not in {'scale','fade'}:raise ValueError('Unsupported titleCollapseMode '+aval)
                        ctl_collapse_mode=aval
                    elif aname=='titleTextEllipsize':
                        if aval not in {'start','middle','end','marquee'}:raise ValueError('Unsupported titleTextEllipsize '+aval)
                    elif aname=='collapsedTitleGravityMode':
                        if aval not in {'entireSpace','availableSpace'}:raise ValueError('Unsupported collapsedTitleGravityMode '+aval)
                    elif aname in {'maxLines','titleMaxLines','subtitleMaxLines'}:
                        if not re.fullmatch(r'\d+',aval) or int(aval)<1:raise ValueError('Invalid '+aname+' '+aval)
                        if aname=='maxLines':ctl_max_lines=int(aval)
                    elif aname=='toolbarId':
                        if aval!='@null' and id_key(aval) is None:raise ValueError('Invalid toolbarId '+aval)
                    elif aname in {'contentScrim','statusBarScrim'} and aval!='@null':
                        if aval.startswith('@drawable/'):
                            if not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',aval):raise ValueError('Invalid '+aname+' '+aval)
                            if aname=='contentScrim' and resource_roots:
                                dpath=find_drawable_xml(aval.split('/')[-1])
                                if dpath is not None:
                                    try:
                                        d_parsed=parse_drawable_element(ET.parse(dpath).getroot(),aval)
                                        if d_parsed['bg_rgba'] is not None and d_parsed['gradient'] is None:
                                            ctl_scrim=rgba_to_csv(d_parsed['bg_rgba'])
                                    except ValueError:
                                        pass
                        else:
                            sc_rgba=static_color_rgba(aval)
                            if aname=='contentScrim':ctl_scrim=rgba_to_csv(sc_rgba)
                    elif aname in {'collapsedTitleTextColor','expandedTitleTextColor','collapsedSubtitleTextColor','expandedSubtitleTextColor'} and aval!='@null':
                        static_color_rgba(aval)
                    elif aname in {'collapsedTitleGravity','expandedTitleGravity'}:
                        gparts=[p.strip() for p in aval.split('|') if p.strip()]
                        if not gparts or any(p not in {'top','bottom','left','right','start','end','center','center_horizontal','center_vertical','fill_vertical'} for p in gparts):
                            raise ValueError('Unsupported '+aname+' '+aval)
                    del n.attrib[k]
            ctl_h='152'
            for sname in style_names:
                if 'Large' in sname or sname=='collapsingToolbarLayoutLargeStyle':
                    ctl_h='152';break
                if 'Medium' in sname or sname=='collapsingToolbarLayoutMediumStyle':
                    ctl_h='112';break
            if A+'layout_height' in n.attrib:
                raw_ctl_h=resolve(n.attrib[A+'layout_height']).strip()
                if raw_ctl_h not in {'match_parent','fill_parent','wrap_content'}:
                    hv=number(raw_ctl_h)
                    if float(hv)>0.0:
                        ctl_h=hv
            collapsing_toolbar_specs[n]=f'{ctl_title}|{ctl_h}|{ctl_scrim}'
            if ctl_collapse_mode!='scale' or ctl_max_lines!=1 or ctl_scrim_dur!=0 or ctl_subtitle!='':
                collapsing_toolbar_ext_specs[n]=f'{ctl_collapse_mode}|{ctl_max_lines}|{ctl_scrim_dur}|{ctl_subtitle}'
        if n.tag in {'androidx.swiperefreshlayout.widget.SwipeRefreshLayout','android.support.v4.widget.SwipeRefreshLayout'}:
            swipe_refresh_nodes.add(n)
            sr_bg=''
            sr_scheme=''
            sr_size='default'
            sr_off_start=-40.0
            sr_off_end=64.0
            sr_has_ext=False
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in SWIPE_REFRESH_ATTRS:
                    aname=k[len(APP):]
                    sr_has_ext=True
                    if aname=='swipeRefreshLayoutProgressSpinnerBackgroundColor':
                        sc=resolve(n.attrib[k]).strip()
                        if sc!='@null':
                            sr_bg=rgba_to_csv(static_color_rgba(sc))
                    elif aname=='swipeRefreshProgressColor':
                        raw_in=n.attrib[k].strip()
                        seen_sr=set()
                        cur=raw_in
                        while cur.startswith('@string/') and cur in pool and cur not in seen_sr:
                            seen_sr.add(cur)
                            cur=pool[cur].strip()
                        if ',' in cur or ';' in cur:
                            tokens=[p.strip() for p in re.split(r'[,;]',cur)]
                        else:
                            resolved=resolve(cur).strip()
                            if resolved=='@null':
                                tokens=None
                            elif resolved.startswith('@array/'):
                                m_arr=re.fullmatch(r'@array/([a-z][a-z0-9_]*)',resolved)
                                if not m_arr or m_arr.group(1) not in arrays:
                                    raise ValueError('Unresolved swipeRefreshProgressColor '+resolved)
                                tokens=[it.strip() for it in arrays[m_arr.group(1)]]
                            else:
                                tokens=[p.strip() for p in re.split(r'[,;]',resolved)]
                        if tokens is not None:
                            if not tokens or any(not tok for tok in tokens):
                                raise ValueError('Invalid swipeRefreshProgressColor '+raw_in)
                            sr_scheme=';'.join(rgba_to_csv(static_color_rgba(tok)) for tok in tokens)
                    elif aname=='swipeRefreshSize':
                        sv=resolve(n.attrib[k]).strip()
                        if sv not in {'default','large'}:
                            raise ValueError('Unsupported swipeRefreshSize '+sv)
                        sr_size=sv
                    elif aname=='swipeRefreshOffsetStart':
                        ov=resolve(n.attrib[k]).strip()
                        if ov!='@null':
                            sr_off_start=float(margin_number(n.attrib[k]))
                    elif aname=='swipeRefreshOffsetEnd':
                        ov=resolve(n.attrib[k]).strip()
                        if ov!='@null':
                            sr_off_end=float(margin_number(n.attrib[k]))
                    del n.attrib[k]
            if sr_has_ext:
                swipe_refresh_ext_specs[n]=f'{sr_bg}|{sr_scheme}|{sr_size}|{fmt_num(sr_off_start)},{fmt_num(sr_off_end)}'
        if n.tag=='androidx.drawerlayout.widget.DrawerLayout':
            drawer_layout_nodes.add(n)
        if n.tag in {'androidx.viewpager2.widget.ViewPager2','androidx.viewpager.widget.ViewPager','android.support.v4.view.ViewPager'}:
            vp_orient=resolve(n.attrib.get(A+'orientation','horizontal')).strip()
            if vp_orient not in {'horizontal','vertical'}:
                raise ValueError('Unsupported ViewPager orientation '+vp_orient)
            if A+'orientation' not in n.attrib:
                n.attrib[A+'orientation']=vp_orient
            view_pager_specs[n]=vp_orient
            vp_user_input=1
            vp_margin=0.0
            vp_transformer='none'
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in VIEW_PAGER_ATTRS:
                    aname=k[len(APP):]
                    aval=resolve(n.attrib[k]).strip()
                    if aname=='userInputEnabled':
                        if aval not in {'true','false'}:
                            raise ValueError('Invalid userInputEnabled '+aval)
                        vp_user_input=1 if aval=='true' else 0
                    elif aname=='pageMargin':
                        if aval!='@null':
                            vp_margin=float(number(n.attrib[k]))
                    elif aname=='pageTransformer':
                        if aval not in {'none','zoom_out','depth'}:
                            raise ValueError('Unsupported pageTransformer '+aval)
                        vp_transformer=aval
                    elif aname=='offscreenPageLimit':
                        if not re.fullmatch(r'-?\d+',aval) or (int(aval)!=-1 and int(aval)<1):
                            raise ValueError('Invalid offscreenPageLimit '+aval)
                    del n.attrib[k]
            vp_orient_code='v' if vp_orient=='vertical' else 'h'
            if vp_orient_code!='h' or vp_user_input!=1 or vp_margin!=0.0 or vp_transformer!='none':
                view_pager_ext_specs[n]=f'{vp_orient_code}|{vp_user_input}|{fmt_num(vp_margin)}|{vp_transformer}'
        if n.tag in {'com.google.android.material.progressindicator.LinearProgressIndicator','com.google.android.material.progressindicator.CircularProgressIndicator','com.google.android.material.progressindicator.LinearWavyProgressIndicator','com.google.android.material.progressindicator.CircularWavyProgressIndicator'}:
            is_linear_pi=n.tag in {'com.google.android.material.progressindicator.LinearProgressIndicator','com.google.android.material.progressindicator.LinearWavyProgressIndicator'}
            if is_linear_pi and A+'orientation' not in n.attrib:
                n.attrib[A+'orientation']='horizontal'
            is_wavy_style=('Wavy' in n.tag) or any('Wavy' in s for s in style_names)
            wavy_wl=None;wavy_amp=None;wavy_speed=None;wavy_stop=None;wavy_gap=None
            has_wavy_attr=False
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in PROGRESS_INDICATOR_ATTRS:
                    aname=k[len(APP):]
                    if aname=='indeterminate' and A+'indeterminate' not in n.attrib:
                        n.attrib[A+'indeterminate']=n.attrib[k]
                    elif aname=='indicatorColor' and A+'progressTint' not in n.attrib and not n.attrib[k].startswith('@array/'):
                        n.attrib[A+'progressTint']=n.attrib[k]
                    elif aname=='trackColor' and A+'progressBackgroundTint' not in n.attrib:
                        n.attrib[A+'progressBackgroundTint']=n.attrib[k]
                    elif aname=='trackCornerRadius':
                        track_corner_radii[n]=number(n.attrib[k])
                    elif aname=='wavelength':
                        wavy_wl=number(n.attrib[k]);has_wavy_attr=True
                    elif aname in {'wavelengthDeterminate','wavelengthIndeterminate'}:
                        wv=number(n.attrib[k]);has_wavy_attr=True
                        if wavy_wl is None:wavy_wl=wv
                    elif aname=='waveAmplitude':
                        wavy_amp=number(n.attrib[k]);has_wavy_attr=True
                    elif aname=='waveSpeed':
                        wavy_speed=margin_number(n.attrib[k]);has_wavy_attr=True
                    elif aname=='trackStopIndicatorSize':
                        wavy_stop=number(n.attrib[k]);has_wavy_attr=True
                    elif aname=='indicatorTrackGapSize':
                        wavy_gap=number(n.attrib[k]);has_wavy_attr=True
                    del n.attrib[k]
            if is_wavy_style or has_wavy_attr:
                has_wave_motion=is_wavy_style or (wavy_wl is not None) or (wavy_amp is not None) or (wavy_speed is not None)
                out_wl=wavy_wl if wavy_wl is not None else (('40.0' if is_linear_pi else '15.0') if has_wave_motion else '0.0')
                out_amp=wavy_amp if wavy_amp is not None else (('3.0' if is_linear_pi else '1.6') if has_wave_motion else '0.0')
                out_speed=wavy_speed if wavy_speed is not None else '0.0'
                out_stop=wavy_stop if wavy_stop is not None else ('4.0' if is_linear_pi else '0.0')
                out_gap=wavy_gap if wavy_gap is not None else '4.0'
                wavy_progress_specs[n]=f'{out_wl}|{out_amp}|{out_speed}|{out_stop}|{out_gap}'
        if n.tag in {'com.google.android.material.slider.Slider','com.google.android.material.slider.RangeSlider'}:
            for src_k,dst_k in ((A+'valueFrom',A+'min'),(A+'valueTo',A+'max'),(A+'value',A+'progress')):
                if src_k in n.attrib:
                    raw_num=resolve(n.attrib.pop(src_k))
                    try:
                        n.attrib[dst_k]=str(int(round(float(raw_num))))
                    except ValueError:
                        raise ValueError('Invalid Slider '+src_k.split('}')[-1]+' '+raw_num)
            sl_step='0.0'
            if A+'stepSize' in n.attrib:
                raw_step=resolve(n.attrib.pop(A+'stepSize'))
                try:
                    sv=float(raw_step)
                except ValueError:
                    raise ValueError('Invalid Slider stepSize '+raw_step)
                if not (sv>=0.0) or sv!=sv or sv==float('inf'):
                    raise ValueError('Invalid Slider stepSize '+raw_step)
                sl_step=str(0.0 if sv==0.0 else sv)
            sl_min_sep='0.0'
            if APP+'minSeparation' in n.attrib:
                raw_ms=resolve(n.attrib[APP+'minSeparation']).strip()
                if raw_ms and raw_ms!='@null':
                    sl_min_sep=number(raw_ms)
            sl_label_beh='floating'
            if APP+'labelBehavior' in n.attrib:
                lb=resolve(n.attrib[APP+'labelBehavior']).strip()
                if lb not in {'floating','withinBounds','gone','visible'}:
                    raise ValueError('Unsupported labelBehavior '+lb)
                sl_label_beh=lb
            sl_expressive='1' if any('Expressive' in s for s in style_names) else '0'
            slider_options_specs[n]=f'{sl_step}|{sl_min_sep}|{sl_label_beh}|{sl_expressive}'
            if n.tag=='com.google.android.material.slider.RangeSlider':
                rs_parsed=None
                if APP+'values' in n.attrib:
                    raw_vals=resolve(n.attrib[APP+'values']).strip()
                    if raw_vals and raw_vals!='@null':
                        def parse_slider_float(tok):
                            m_num=re.fullmatch(r'(-?(?:\d+(?:\.\d+)?|\.\d+))[fF]?',tok)
                            if not m_num:raise ValueError('Invalid RangeSlider values '+raw_vals)
                            fv=float(m_num.group(1))
                            if fv!=fv or fv in (float('inf'),float('-inf')):raise ValueError('Invalid RangeSlider values '+raw_vals)
                            return fv
                        if raw_vals.startswith('@array/'):
                            m_arr=re.fullmatch(r'@array/([a-z][a-z0-9_]*)',raw_vals)
                            if not m_arr:raise ValueError('Invalid RangeSlider values '+raw_vals)
                            arr_name=m_arr.group(1)
                            if arr_name in arrays:
                                arr_items=[resolve(it).strip() for it in arrays[arr_name]]
                                if arr_items:
                                    rs_parsed=[parse_slider_float(tok) for tok in arr_items]
                        else:
                            inner=raw_vals
                            if inner.startswith('[') or inner.endswith(']'):
                                if not (inner.startswith('[') and inner.endswith(']')):
                                    raise ValueError('Invalid RangeSlider values '+raw_vals)
                                inner=inner[1:-1].strip()
                            if not inner:raise ValueError('Invalid RangeSlider values '+raw_vals)
                            parts=[p.strip() for p in inner.split(',')]
                            if any(not p for p in parts):raise ValueError('Invalid RangeSlider values '+raw_vals)
                            rs_parsed=[parse_slider_float(tok) for tok in parts]
                range_slider_specs[n]=rs_parsed
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in SLIDER_ATTRS:
                    del n.attrib[k]
        if n.tag=='androidx.constraintlayout.widget.Guideline':
            guide_orient=resolve(n.attrib.get(A+'orientation','horizontal'))
            if guide_orient not in {'vertical','horizontal'}:raise ValueError('Unsupported Guideline orientation '+guide_orient)
            if A+'visibility' not in n.attrib:n.attrib[A+'visibility']='gone'
            n.attrib[A+'layout_width']='0dp'; n.attrib[A+'layout_height']='0dp'
            guide_begin=n.attrib.pop(APP+'layout_constraintGuide_begin',None)
            guide_end=n.attrib.pop(APP+'layout_constraintGuide_end',None)
            guide_percent=n.attrib.pop(APP+'layout_constraintGuide_percent',None)
            begin_val=margin_number(guide_begin) if guide_begin is not None else None
            end_val=margin_number(guide_end) if guide_end is not None else None
            pct_val=None
            if guide_percent is not None:
                raw_pct=resolve(guide_percent)
                try: pct=float(raw_pct)
                except ValueError: raise ValueError('Invalid Guideline percent '+raw_pct)
                if not 0.0<=pct<=1.0:raise ValueError('Invalid Guideline percent '+raw_pct)
                pct_val=str(pct)
            guideline_specs[n]=(guide_orient,begin_val,end_val,pct_val)
        if n.tag=='androidx.constraintlayout.widget.Barrier':
            if A+'visibility' not in n.attrib:n.attrib[A+'visibility']='gone'
            n.attrib[A+'layout_width']='0dp'; n.attrib[A+'layout_height']='0dp'
            bdir='left'
            if APP+'barrierDirection' in n.attrib:
                bdir=resolve(n.attrib.pop(APP+'barrierDirection'))
                if bdir not in {'left','right','top','bottom','start','end'}:raise ValueError('Unsupported Barrier direction '+bdir)
            allows_gone=True
            if APP+'barrierAllowsGoneWidgets' in n.attrib:
                bag=resolve(n.attrib.pop(APP+'barrierAllowsGoneWidgets')).strip()
                if bag not in {'true','false'}:raise ValueError('Invalid barrierAllowsGoneWidgets '+bag)
                allows_gone=(bag=='true')
            bmargin=0.0
            if APP+'barrierMargin' in n.attrib:
                bmargin=float(margin_number(n.attrib.pop(APP+'barrierMargin')))
            brefs=parse_referenced_ids(n.attrib.pop(APP+'constraint_referenced_ids',''))
            base_barrier=bdir+':'+','.join(brefs)
            if bmargin==0.0 and allows_gone:
                barrier_specs[n]=base_barrier
            else:
                barrier_specs[n]=f'{base_barrier}|{fmt_num(bmargin)}|{"1" if allows_gone else "0"}'
        if n.tag=='androidx.constraintlayout.widget.Group':
            if A+'visibility' not in n.attrib:n.attrib[A+'visibility']='gone'
            n.attrib[A+'layout_width']='0dp'; n.attrib[A+'layout_height']='0dp'
            grefs=parse_referenced_ids(n.attrib.pop(APP+'constraint_referenced_ids',''))
            group_specs[n]=','.join(grefs)
        if n.tag in {'androidx.constraintlayout.helper.widget.Layer','android.support.constraint.helper.Layer'}:
            lrefs=parse_referenced_ids(n.attrib.pop(APP+'constraint_referenced_ids',''))
            layer_specs[n]=','.join(lrefs)
        if n.tag in {'androidx.constraintlayout.widget.Placeholder','android.support.constraint.Placeholder'}:
            empty_vis='gone'
            for ev_key in ('placeholder_emptyVisibility','emptyVisibility'):
                if APP+ev_key in n.attrib:
                    empty_vis=resolve(n.attrib.pop(APP+ev_key)).strip()
                    if empty_vis not in {'visible','invisible','gone'}:raise ValueError('Unsupported '+ev_key+' '+empty_vis)
            if A+'visibility' not in n.attrib:n.attrib[A+'visibility']=empty_vis
            ph_content=''
            if APP+'content' in n.attrib:
                raw_c=resolve(n.attrib.pop(APP+'content')).strip()
                if raw_c and raw_c!='@null':
                    cid=id_key(raw_c)
                    if cid is None:raise ValueError('Invalid Placeholder content '+raw_c)
                    ph_content=cid
            placeholder_specs[n]=f'{ph_content}|{empty_vis}'
        if n.tag in {'androidx.constraintlayout.helper.widget.Flow','android.support.constraint.helper.Flow'}:
            if A+'visibility' not in n.attrib:n.attrib[A+'visibility']='gone'
            flow_orient_raw=resolve(n.attrib.get(A+'orientation','horizontal')).strip()
            if flow_orient_raw not in {'horizontal','vertical'}:raise ValueError('Unsupported Flow orientation '+flow_orient_raw)
            flow_orient='1' if flow_orient_raw=='vertical' else '0'
            flow_wrap='none'
            flow_hgap='0'
            flow_vgap='0'
            flow_max_wrap='0'
            flow_hstyle='spread'
            flow_vstyle='spread'
            flow_hbias=0.5
            flow_vbias=0.5
            flow_halign='center'
            flow_valign='center'
            flow_ext_slots=['']*8
            has_flow_ext=False
            flow_refs=parse_referenced_ids(n.attrib.pop(APP+'constraint_referenced_ids')) if APP+'constraint_referenced_ids' in n.attrib else []
            for k in list(n.attrib):
                if not k.startswith(APP):continue
                aname=k[len(APP):]
                if aname=='flow_horizontalGap':
                    flow_hgap=margin_number(n.attrib.pop(k))
                elif aname=='flow_verticalGap':
                    flow_vgap=margin_number(n.attrib.pop(k))
                elif aname=='flow_wrapMode':
                    wm=resolve(n.attrib.pop(k)).strip()
                    if wm not in {'none','chain','aligned','chain2'}:raise ValueError('Unsupported flow_wrapMode '+wm)
                    flow_wrap='chain' if wm=='chain2' else wm
                elif aname in {'flow_horizontalStyle','flow_verticalStyle','flow_firstHorizontalStyle','flow_firstVerticalStyle','flow_lastHorizontalStyle','flow_lastVerticalStyle'}:
                    fs=resolve(n.attrib.pop(k)).strip()
                    if fs not in {'spread','spread_inside','packed'}:raise ValueError('Unsupported '+aname+' '+fs)
                    if aname=='flow_horizontalStyle':flow_hstyle=fs
                    elif aname=='flow_verticalStyle':flow_vstyle=fs
                    else:
                        flow_ext_slots[{'flow_firstHorizontalStyle':0,'flow_lastHorizontalStyle':1,'flow_firstVerticalStyle':4,'flow_lastVerticalStyle':5}[aname]]=fs
                        has_flow_ext=True
                elif aname in {'flow_horizontalBias','flow_verticalBias','flow_firstHorizontalBias','flow_firstVerticalBias','flow_lastHorizontalBias','flow_lastVerticalBias'}:
                    rb=resolve(n.attrib.pop(k)).strip()
                    try: bv=float(rb)
                    except ValueError: raise ValueError('Invalid '+aname+' '+rb)
                    if not 0.0<=bv<=1.0:raise ValueError('Invalid '+aname+' '+rb)
                    if aname=='flow_horizontalBias':flow_hbias=bv
                    elif aname=='flow_verticalBias':flow_vbias=bv
                    else:
                        flow_ext_slots[{'flow_firstHorizontalBias':2,'flow_lastHorizontalBias':3,'flow_firstVerticalBias':6,'flow_lastVerticalBias':7}[aname]]=fmt_num(bv)
                        has_flow_ext=True
                elif aname=='flow_horizontalAlign':
                    ha=resolve(n.attrib.pop(k)).strip()
                    if ha not in {'start','end','center'}:raise ValueError('Unsupported flow_horizontalAlign '+ha)
                    flow_halign=ha
                elif aname=='flow_verticalAlign':
                    va=resolve(n.attrib.pop(k)).strip()
                    if va not in {'top','bottom','center','baseline','start','end'}:raise ValueError('Unsupported flow_verticalAlign '+va)
                    flow_valign=va
                elif aname=='flow_maxElementsWrap':
                    rm=resolve(n.attrib.pop(k)).strip()
                    if not re.fullmatch(r'-?\d+',rm) or int(rm)<-1:raise ValueError('Invalid flow_maxElementsWrap '+rm)
                    flow_max_wrap=str(max(0,int(rm)))
            base_flow=f'{flow_orient}|{flow_wrap}|{flow_hgap}|{flow_vgap}|{flow_max_wrap}|'+','.join(flow_refs)
            if (flow_hstyle!='spread' or flow_vstyle!='spread' or flow_hbias!=0.5 or flow_vbias!=0.5 or flow_halign!='center' or flow_valign!='center'):
                flow_specs[n]=f'{base_flow}|{flow_hstyle},{flow_vstyle},{fmt_num(flow_hbias)},{fmt_num(flow_vbias)},{flow_halign},{flow_valign},{flow_max_wrap}'
            else:
                flow_specs[n]=base_flow
            if has_flow_ext:
                flow_ext_specs[n]=','.join(flow_ext_slots)
        if n.tag in {'androidx.recyclerview.widget.RecyclerView','android.support.v7.widget.RecyclerView','androidx.preference.PreferenceRecyclerView','HorizontalScrollView','android.widget.HorizontalScrollView','LinearLayout','android.widget.LinearLayout','androidx.appcompat.widget.LinearLayoutCompat','androidx.appcompat.widget.FitWindowsLinearLayout'}:
            raw_car_lm=resolve(n.attrib.get(APP+'layoutManager','')).strip()
            has_carousel=('CarouselLayoutManager' in raw_car_lm) or any(APP+ck in n.attrib for ck in CAROUSEL_ATTRS)
            car_strategy='multi_browse'
            car_spacing=8.0
            car_align='start'
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in CAROUSEL_ATTRS:
                    aname=k[len(APP):]
                    aval=resolve(n.attrib[k]).strip()
                    if aname=='carousel_strategy':
                        if aval not in {'multi_browse','hero','uncontained','full_screen'}:
                            raise ValueError('Unsupported carousel_strategy '+aval)
                        car_strategy=aval
                    elif aname=='carousel_alignment':
                        if aval not in {'start','center'}:
                            raise ValueError('Unsupported carousel_alignment '+aval)
                        car_align=aval
                    elif aname=='carousel_itemSpacing':
                        if aval!='@null':
                            car_spacing=float(number(n.attrib[k]))
                    del n.attrib[k]
            if has_carousel:
                carousel_layout_specs[n]=f'{car_strategy}|{fmt_num(car_spacing)}|{car_align}'
        if n.tag in {'androidx.recyclerview.widget.RecyclerView','android.support.v7.widget.RecyclerView','androidx.preference.PreferenceRecyclerView'}:
            raw_lm=resolve(n.attrib.get(APP+'layoutManager','')).strip()
            if 'StaggeredGridLayoutManager' in raw_lm:
                rv_mgr='staggered'
            elif 'GridLayoutManager' in raw_lm:
                rv_mgr='grid'
            else:
                rv_mgr='linear'
            is_grid=rv_mgr in {'grid','staggered'}
            raw_orient=resolve(n.attrib.get(A+'orientation','vertical')).strip()
            if raw_orient not in {'vertical','horizontal'}:
                raise ValueError('Unsupported orientation')
            rv_orient='h' if raw_orient=='horizontal' else 'v'
            rv_span=1
            rv_reverse=0
            rv_stack_end=0
            n.tag='GridView' if is_grid else 'ListView'
            for k in list(n.attrib):
                if k.startswith(APP) and k[len(APP):] in RECYCLER_VIEW_ATTRS:
                    aname=k[len(APP):]
                    aval=resolve(n.attrib[k]).strip()
                    if aname=='spanCount':
                        if not re.fullmatch(r'\d+',aval) or not 1<=int(aval)<=256:
                            raise ValueError('Invalid spanCount '+aval)
                        rv_span=int(aval)
                        if is_grid and A+'numColumns' not in n.attrib:
                            n.attrib[A+'numColumns']=str(rv_span)
                    elif aname=='reverseLayout':
                        if aval not in {'true','false'}:
                            raise ValueError('Invalid reverseLayout '+aval)
                        rv_reverse=1 if aval=='true' else 0
                    elif aname=='stackFromEnd':
                        if aval not in {'true','false'}:
                            raise ValueError('Invalid stackFromEnd '+aval)
                        rv_stack_end=1 if aval=='true' else 0
                    del n.attrib[k]
            if rv_mgr=='staggered' or rv_orient=='h' or rv_reverse==1 or rv_stack_end==1:
                recycler_layout_ext_specs[n]=f'{rv_mgr}|{rv_orient}|{rv_span}|{rv_reverse}|{rv_stack_end}'
        if n.tag=='androidx.preference.internal.PreferenceImageView':
            for dk in ('maxWidth','maxHeight'):
                if APP+dk in n.attrib:
                    val=n.attrib.pop(APP+dk)
                    if A+dk not in n.attrib:n.attrib[A+dk]=val
        if n.tag in {'androidx.fragment.app.FragmentContainerView','fragment'}:
            n.attrib.pop(A+'name',None); n.attrib.pop(A+'tag',None); n.attrib.pop('class',None)
            n.attrib.pop(APP+'defaultNavHost',None); n.attrib.pop(APP+'navGraph',None)
        if n.tag in {'ViewStub','android.view.ViewStub'}:
            if A+'visibility' not in n.attrib:n.attrib[A+'visibility']='gone'
            n.attrib.pop(A+'layout',None); n.attrib.pop(A+'inflatedId',None)
        n.tag=canonical_view_tag(n.tag)
        children=list(leading_children)
        for child in n:children.extend(expand(child,stack))
        n[:]=children
        return [n]
    images={};indicator_tints={};vector_tint_modes={}
    def resolve(v,stack=()):
        if v=='@null':return '@null'
        if v in ANDROID_COLORS:return ANDROID_COLORS[v]
        if v in ANDROID_STRINGS:return ANDROID_STRINGS[v]
        if v in ANDROID_DIMENS:return ANDROID_DIMENS[v]
        if v in ANDROID_INTEGERS:return ANDROID_INTEGERS[v]
        if v.startswith('?'):
            tval=lookup_theme_attr(v)
            if tval is not None:
                if v in stack:raise ValueError('Cyclic resource reference: '+' -> '.join(stack+(v,)))
                return resolve(tval,stack+(v,))
            return v
        if re.fullmatch(r'@color/[a-z][a-z0-9_]*',v) and resource_roots:
            cidx,_=find_color_xml(v.split('/')[-1])
            if cidx is not None:
                if v in pool_root and pool_root[v]==cidx:raise ValueError('Ambiguous color resource '+v)
                if v not in pool_root or cidx<pool_root[v]:return v
        if v.startswith('@') and not v.startswith(('@id/','@+id/','@android:id/','@drawable/','@android:drawable/','@anim/','@android:anim/','@animator/','@android:animator/','@interpolator/','@android:interpolator/','@array/','@style/','@android:style/','@layout/','@font/','@menu/','@navigation/','@xml/')):
            if v in stack:raise ValueError('Cyclic resource reference: '+' -> '.join(stack+(v,)))
            if v in pool:return resolve(pool[v],stack+(v,))
            if v in BUILTIN_LIBRARY_DIMENS:return resolve(BUILTIN_LIBRARY_DIMENS[v],stack+(v,))
            if v in BUILTIN_LIBRARY_COLORS:return resolve(BUILTIN_LIBRARY_COLORS[v],stack+(v,))
            raise ValueError('Unresolved resource '+v)
        return v
    def theme_token(v):
        parsed=parse_theme_ref(v)
        return parsed[1] if parsed is not None else None
    def number(v):
        v=resolve(v)
        tok=theme_token(v)
        if tok is not None:
            if tok not in THEME_DIMENS:raise ValueError('Unresolved theme dimension '+v)
            v=THEME_DIMENS[tok]
        if not re.fullmatch(r'(?:\d+(?:\.\d+)?|\.\d+)(?:dp|dip|sp|px)?',v):raise ValueError('Expected dp/sp/px number: '+v)
        return str(float(re.sub(r'(?:dp|dip|sp|px)$','',v)))
    def margin_number(v):
        v=resolve(v)
        tok=theme_token(v)
        if tok is not None:
            if tok not in THEME_DIMENS:raise ValueError('Unresolved theme dimension '+v)
            v=THEME_DIMENS[tok]
        if not re.fullmatch(r'-?(?:\d+(?:\.\d+)?|\.\d+)(?:dp|dip|sp|px)?',v):raise ValueError('Expected margin in dp/sp/px: '+v)
        return str(float(re.sub(r'(?:dp|dip|sp|px)$','',v)))
    def size(v):
        if v in {'match_parent','fill_parent'}:return 'Size::Match'
        if v=='wrap_content':return 'Size::Wrap'
        return 'Size::Dp('+number(v)+')'
    def parse_color_rgba(v):
        v=resolve(v)
        tok=theme_token(v)
        if tok is not None:
            if tok not in THEME_COLORS:raise ValueError('Unresolved theme color '+v)
            v=THEME_COLORS[tok]
        if not re.fullmatch(r'#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})',v):raise ValueError('Expected #RGB, #ARGB, #RRGGBB or #AARRGGBB: '+v)
        raw=v[1:]
        if len(raw) in (3,4):raw=''.join(ch*2 for ch in raw)
        a=int(raw[:2],16) if len(raw)==8 else 255;raw=raw[-6:]
        return (int(raw[0:2],16),int(raw[2:4],16),int(raw[4:6],16),a)
    def rgba_to_rust(rgba):
        return 'Some(Color('+','.join(str(c) for c in rgba)+'))'
    def rgba_to_csv(rgba):
        return ','.join(str(c) for c in rgba)
    def color(v):
        return rgba_to_rust(parse_color_rgba(v))
    def tint_selector(v, compact=False):
        if not re.fullmatch(r'@color/[a-z][a-z0-9_]*',v):return None
        if not resource_roots:raise ValueError('Color selector needs --resources')
        _,file=find_color_xml(v.split('/')[-1])
        if file is None:raise ValueError('Unresolved color selector '+v)
        root=ET.parse(file).getroot()
        if root.tag!='selector' or any(not k.startswith(TOOLS) for k in root.attrib):raise ValueError('Unsupported color selector '+v)
        flags={'state_enabled':1,'state_pressed':2,'state_focused':4,'state_selected':8,'state_activated':16,'state_hovered':32,'state_checked':64,'state_dragged':128}
        if compact:flags['state_checkable']=256
        rules=[]
        for item in root:
            if item.tag!='item' or len(item) or A+'color' not in item.attrib:raise ValueError('Invalid color selector item '+v)
            required=forbidden=0
            for attribute,raw in item.attrib.items():
                if attribute in {A+'color',A+'alpha'} or attribute.startswith(TOOLS):continue
                name=attribute[len(A):] if attribute.startswith(A) else attribute[len(APP):] if attribute.startswith(APP) and attribute[len(APP):]=='state_dragged' else ''
                if name not in flags:raise ValueError('Unsupported color selector state '+attribute)
                value=resolve(raw)
                if value not in {'true','false'}:raise ValueError('Invalid color selector boolean '+value)
                if value=='true':required|=flags[name]
                else:forbidden|=flags[name]
            raw_color=resolve(item.attrib[A+'color'])
            token=raw_color.removeprefix('?attr/') if raw_color.startswith('?attr/') else None
            if token is not None and (not compact or token not in CARD_THEME_COLORS):raise ValueError('Unsupported color selector theme attribute '+raw_color)
            rgba=None if token is not None else color(raw_color)[len('Some(Color('):-2].split(',')
            alpha_factor=1.0
            if A+'alpha' in item.attrib:
                raw=resolve(item.attrib[A+'alpha'])
                if not re.fullmatch(r'(?:0(?:\.\d+)?|1(?:\.0+)?|\.\d+)',raw) or not 0<=float(raw)<=1:raise ValueError('Invalid color selector alpha '+raw)
                alpha_factor=float(raw)
            if token is not None:
                rules.append(','.join([str(required),str(forbidden),'@'+token,str(int(255*alpha_factor+0.5))]))
            else:
                rgba[3]=str(int(int(rgba[3])*alpha_factor+0.5))
                rules.append(','.join([str(required),str(forbidden)]+rgba) if compact else 'TintState { required:'+str(required)+', forbidden:'+str(forbidden)+', color:Color('+','.join(rgba)+') }')
        if not rules:raise ValueError('Empty color selector '+v)
        return rules if compact else '&['+','.join(rules)+']'
    def static_color_rgba(v):
        rv=resolve(v)
        if re.fullmatch(r'@color/[a-z][a-z0-9_]*',rv) and resource_roots:
            _,cfile=find_color_xml(rv.split('/')[-1])
            if cfile is not None:
                croot=ET.parse(cfile).getroot()
                if croot.tag!='selector':raise ValueError('Unsupported color selector '+rv)
                chosen=None;first_item=None
                for item in croot:
                    if item.tag!='item' or A+'color' not in item.attrib:continue
                    if first_item is None:first_item=item
                    state_attrs=[k for k in item.attrib if k not in {A+'color',A+'alpha'} and not k.startswith(TOOLS)]
                    if not state_attrs:chosen=item;break
                target_item=chosen if chosen is not None else first_item
                if target_item is None:raise ValueError('Empty color selector '+rv)
                r,g,b,al=parse_color_rgba(target_item.attrib[A+'color'])
                if A+'alpha' in target_item.attrib:
                    af=float(resolve(target_item.attrib[A+'alpha']))
                    al=int(al*af+0.5)
                return (r,g,b,al)
        return parse_color_rgba(rv)
    def strip_tools(attrib):
        return {k:val for k,val in attrib.items() if not k.startswith(TOOLS)}
    def parse_fraction(raw):
        v=resolve(raw).strip()
        if v.endswith('%'):
            try: val=float(v[:-1])/100.0
            except ValueError: raise ValueError('Invalid gradient center position '+raw)
        else:
            try: val=float(v)
            except ValueError: raise ValueError('Invalid gradient center position '+raw)
        if not 0.0<=val<=1.0:raise ValueError('Invalid gradient center position '+raw)
        return 0.0 if val==0.0 else val
    def resolve_and_rasterize_vector(vector_source, name):
        orig_root=vector_source if isinstance(vector_source,ET.Element) else ET.parse(vector_source).getroot()
        if orig_root.tag!='vector':raise ValueError('Expected vector drawable: '+str(vector_source))
        def clone_elem(el):
            c=ET.Element(el.tag,{k:val for k,val in el.attrib.items() if not k.startswith(TOOLS)})
            for child in el:c.append(clone_elem(child))
            return c
        root=clone_elem(orig_root)
        auto_mirrored=False
        if A+'autoMirrored' in root.attrib:
            am=resolve(root.attrib[A+'autoMirrored']).strip()
            if am not in {'true','false'}:raise ValueError('Invalid vector autoMirrored '+am)
            root.attrib[A+'autoMirrored']=am
            auto_mirrored=(am=='true')
        if A+'tintMode' in root.attrib:
            root.attrib[A+'tintMode']=resolve(root.attrib[A+'tintMode']).strip()
        vec_w=vec_h=24.0
        for dim_attr in ('width','height'):
            if A+dim_attr in root.attrib:
                val=float(number(root.attrib[A+dim_attr]))
                root.attrib[A+dim_attr]=f'{val}dp'
                if dim_attr=='width':vec_w=val
                else:vec_h=val
        for vp_attr in ('viewportWidth','viewportHeight','alpha'):
            if A+vp_attr in root.attrib:root.attrib[A+vp_attr]=resolve(root.attrib[A+vp_attr])
        resolved_tint=None;raw_tint_rgba=None
        if A+'tint' in root.attrib:
            raw_tint=root.attrib[A+'tint']
            if raw_tint in ('?attr/colorControlNormal','?android:attr/colorControlNormal'):
                resolved_tint='controlNormal'
                raw_tint_rgba=static_color_rgba(raw_tint)
            else:
                try: raw_tint_rgba=static_color_rgba(raw_tint)
                except ValueError: raise ValueError('Unsupported vector tint '+raw_tint)
                r,g,b,al=raw_tint_rgba
                resolved_tint=f'#{al:02x}{r:02x}{g:02x}{b:02x}'
                root.attrib[A+'tint']=resolved_tint
        first_color_rgba=None
        for el in root.iter():
            if el is root:continue
            if A+'pathData' in el.attrib:
                el.attrib[A+'pathData']=resolve(el.attrib[A+'pathData']).strip()
            for c_attr in ('fillColor','strokeColor','startColor','centerColor','endColor','color'):
                if A+c_attr in el.attrib:
                    rgba=static_color_rgba(el.attrib[A+c_attr])
                    if first_color_rgba is None and rgba[3]>0:first_color_rgba=rgba
                    r,g,b,al=rgba
                    el.attrib[A+c_attr]=f'#{r:02x}{g:02x}{b:02x}' if al==255 else f'#{al:02x}{r:02x}{g:02x}{b:02x}'
            for n_attr in ('fillAlpha','strokeAlpha','strokeWidth','strokeMiterLimit','trimPathStart','trimPathEnd','trimPathOffset','rotation','pivotX','pivotY','scaleX','scaleY','translateX','translateY','startX','startY','endX','endY','centerX','centerY','gradientRadius','offset','type','tileMode'):
                if A+n_attr in el.attrib:el.attrib[A+n_attr]=resolve(el.attrib[A+n_attr]).strip()
        with tempfile.TemporaryDirectory() as td:
            tmp_vec=Path(td)/'vector.xml'
            ET.ElementTree(root).write(tmp_vec,encoding='utf-8',xml_declaration=True)
            if name not in images:images[name]=rasterize_vector(tmp_vec,density)
            vt=vector_tint(tmp_vec)
            vtm=vector_tint_mode(tmp_vec)
            auto_mirrored=vector_auto_mirrored(tmp_vec)
        vector_tint_modes[name]=vtm
        if resolved_tint is not None:indicator_tints[name]=resolved_tint
        elif vt is not None:indicator_tints[name]=vt
        return vec_w,vec_h,raw_tint_rgba,first_color_rgba,auto_mirrored
    def decode_drawable_image(file_path, source_density):
        scale=160.0/source_density
        if is_nine_patch(file_path):
            w,h,pixels,pad_px,x_bands,y_bands=decode_nine_patch_bands(file_path)
            pad_dp=[(p*160.0)/source_density for p in pad_px]
            stretch_spec=','.join(f'{a},{b}' for a,b in x_bands)+';'+','.join(f'{a},{b}' for a,b in y_bands)
            return w,h,pixels,scale,pad_dp,stretch_spec
        w,h,pixels=decode_png(file_path)
        return w,h,pixels,scale,None,''
    def validate_animation_list(root, ref_label):
        ala=strip_tools(root.attrib)
        if set(ala)-{A+'oneshot',A+'variablePadding',A+'visible'}:raise ValueError('Unsupported animation-list attribute: '+ref_label)
        for bk in (A+'oneshot',A+'variablePadding',A+'visible'):
            if bk in ala and resolve(ala[bk]).strip() not in {'true','false'}:raise ValueError('Invalid animation-list boolean '+ala[bk])
        items=list(root)
        if not items or any(it.tag!='item' for it in items):raise ValueError('Invalid animation-list drawable: '+ref_label)
        for it in items:
            ia=strip_tools(it.attrib)
            if set(ia)-{A+'drawable',A+'duration'}:raise ValueError('Unsupported animation-list item attribute: '+ref_label)
            if A+'duration' in ia:
                dv=resolve(ia[A+'duration']).strip()
                if not re.fullmatch(r'\d+',dv):raise ValueError('Invalid animation-list duration '+dv)
        return items[0]
    def validate_drawable_wrapper(root, ref_label):
        wa=strip_tools(root.attrib)
        allowed_wrap={
            'rotate':{A+'drawable',A+'fromDegrees',A+'toDegrees',A+'pivotX',A+'pivotY',A+'visible',A+'autoMirrored'},
            'scale':{A+'drawable',A+'scaleWidth',A+'scaleHeight',A+'scaleGravity',A+'useIntrinsicSizeAsMinimum',A+'level',A+'visible',A+'autoMirrored'},
            'clip':{A+'drawable',A+'clipOrientation',A+'gravity',A+'visible',A+'autoMirrored'},
        }[root.tag]
        if set(wa)-allowed_wrap:raise ValueError('Unsupported '+root.tag+' drawable attribute: '+ref_label)
        for bk in (A+'visible',A+'useIntrinsicSizeAsMinimum',A+'autoMirrored'):
            if bk in wa and resolve(wa[bk]).strip() not in {'true','false'}:raise ValueError('Invalid '+root.tag+' boolean '+wa[bk])
        if root.tag=='rotate':
            for dk in (A+'fromDegrees',A+'toDegrees'):
                if dk in wa:
                    dv=resolve(wa[dk]).strip()
                    if not re.fullmatch(r'-?(?:\d+(?:\.\d+)?|\.\d+)',dv):raise ValueError('Invalid rotate drawable degrees '+dv)
            for pk in (A+'pivotX',A+'pivotY'):
                if pk in wa:
                    pv=resolve(wa[pk]).strip()
                    if not re.fullmatch(r'-?(?:\d+(?:\.\d+)?|\.\d+)(?:%p|%P|%|dp|dip|px)?',pv):raise ValueError('Invalid rotate drawable pivot '+pv)
        elif root.tag=='scale':
            for sk in (A+'scaleWidth',A+'scaleHeight'):
                if sk in wa:
                    sv=resolve(wa[sk]).strip()
                    if not re.fullmatch(r'-?(?:\d+(?:\.\d+)?|\.\d+)(?:%p|%)?',sv):raise ValueError('Invalid scale drawable value '+sv)
            if A+'scaleGravity' in wa:
                gp=[p.strip() for p in resolve(wa[A+'scaleGravity']).split('|') if p.strip()]
                if not gp or any(p not in BITMAP_GRAVITY_TOKENS for p in gp):raise ValueError('Unsupported scaleGravity '+wa[A+'scaleGravity'])
            if A+'level' in wa:
                lv=resolve(wa[A+'level']).strip()
                if not re.fullmatch(r'-?\d+',lv):raise ValueError('Invalid scale drawable level '+lv)
        elif root.tag=='clip':
            if A+'clipOrientation' in wa:
                co=resolve(wa[A+'clipOrientation']).strip()
                if co not in {'horizontal','vertical'}:raise ValueError('Unsupported clipOrientation '+co)
            if A+'gravity' in wa:
                gp=[p.strip() for p in resolve(wa[A+'gravity']).split('|') if p.strip()]
                if not gp or any(p not in BITMAP_GRAVITY_TOKENS for p in gp):raise ValueError('Unsupported clip gravity '+wa[A+'gravity'])
        auto_m=resolve(wa[A+'autoMirrored']).strip()=='true' if A+'autoMirrored' in wa else False
        if A+'drawable' in wa and not len(root):
            return ('ref',resolve(wa[A+'drawable']),auto_m)
        if A+'drawable' not in wa and len(root)==1:
            return ('child',root[0],auto_m)
        raise ValueError('Invalid '+root.tag+' drawable: '+ref_label)
    def select_list_drawable_item(root, ref_label):
        la=strip_tools(root.attrib)
        allowed_root=({A+'visible',A+'autoMirrored',A+'constantSize',A+'variablePadding',A+'dither'} if root.tag=='level-list'
            else {A+'visible',A+'autoMirrored',A+'paddingMode',A+'opacity',A+'dither'})
        if set(la)-allowed_root:raise ValueError('Unsupported '+root.tag+' attribute: '+ref_label)
        for bk in (A+'visible',A+'autoMirrored',A+'constantSize',A+'variablePadding',A+'dither'):
            if bk in la and resolve(la[bk]).strip() not in {'true','false'}:raise ValueError('Invalid '+root.tag+' boolean '+la[bk])
        items=list(root)
        if not items or any(it.tag!='item' for it in items):raise ValueError('Invalid '+root.tag+' drawable: '+ref_label)
        allowed_item=({A+'drawable',A+'minLevel',A+'maxLevel'} if root.tag=='level-list'
            else {A+'drawable',A+'id',A+'left',A+'top',A+'right',A+'bottom',A+'start',A+'end',A+'width',A+'height',A+'gravity'})
        chosen=items[0]
        matched_zero=False
        for it in items:
            ia=strip_tools(it.attrib)
            if set(ia)-allowed_item:raise ValueError('Unsupported '+root.tag+' item attribute: '+ref_label)
            if not ((A+'drawable' in ia and not len(it)) or (A+'drawable' not in ia and len(it)==1)):
                raise ValueError('Invalid '+root.tag+' item: '+ref_label)
            if root.tag=='level-list':
                raw_min=resolve(ia[A+'minLevel']).strip() if A+'minLevel' in ia else '0'
                raw_max=resolve(ia[A+'maxLevel']).strip() if A+'maxLevel' in ia else '0'
                if not re.fullmatch(r'-?\d+',raw_min):raise ValueError('Invalid level-list level '+raw_min)
                if not re.fullmatch(r'-?\d+',raw_max):raise ValueError('Invalid level-list level '+raw_max)
                if not matched_zero and int(raw_min)<=0<=int(raw_max):
                    chosen=it
                    matched_zero=True
        auto_m=resolve(la[A+'autoMirrored']).strip()=='true' if A+'autoMirrored' in la else False
        cia=strip_tools(chosen.attrib)
        if A+'drawable' in cia and not len(chosen):
            return ('ref',resolve(cia[A+'drawable']),auto_m)
        return ('child',chosen[0],auto_m)
    def empty_drawable_dict():
        return {'bg':'None','bg_rgba':None,'radius':'0.0','corner_radii':None,'gradient':None,'stroke':'None','stroke_rgba':None,'stroke_width':'0.0','padding':None,'size':[0.0,0.0],'layers':[],'stateful_layers':[],'ripple':None,'bg_selector':None,'bitmap_tint':None,'shape_kind':None,'bg_image':None,'stateful_bg_images':[],'auto_mirrored':False,'animation_list':None,'dynamic_drawable':None,'animated_transitions':[],'animated_vector':None}
    def rasterize_shape_to_image(xroot, dref, target_name):
        sd=parse_shape_element(xroot,dref)
        sw_dp=sd['size'][0] if sd['size'][0]>0 else 24.0
        sh_dp=sd['size'][1] if sd['size'][1]>0 else 24.0
        vec=ET.Element('vector',{A+'width':f'{sw_dp}dp',A+'height':f'{sh_dp}dp',A+'viewportWidth':str(sw_dp),A+'viewportHeight':str(sh_dp)})
        stw=float(sd['stroke_width']) if sd['stroke_rgba'] is not None else 0.0
        ins=stw*0.5;iw=max(0.1,sw_dp-stw);ih=max(0.1,sh_dp-stw)
        if sd['shape_kind']=='line':
            cy=sh_dp*0.5
            pdata=f'M0,{cy} L{sw_dp},{cy}'
        elif sd['radius']=='-1.0':
            rx=iw*0.5;ry=ih*0.5;cx=sw_dp*0.5;cy=sh_dp*0.5
            pdata=f'M{cx},{cy-ry} A{rx},{ry} 0 1,0 {cx},{cy+ry} A{rx},{ry} 0 1,0 {cx},{cy-ry} Z'
        else:
            if sd['corner_radii'] is not None:
                tl,tr,br,bl=[min(float(x),iw*0.5,ih*0.5) for x in sd['corner_radii'].split(',')]
            else:
                rv=min(max(0.0,float(sd['radius'])),iw*0.5,ih*0.5)
                tl=tr=br=bl=rv
            x0,y0,x1,y1=ins,ins,sw_dp-ins,sh_dp-ins
            pdata=f'M{x0+tl},{y0} L{x1-tr},{y0} A{tr},{tr} 0 0,1 {x1},{y0+tr} L{x1},{y1-br} A{br},{br} 0 0,1 {x1-br},{y1} L{x0+bl},{y1} A{bl},{bl} 0 0,1 {x0},{y1-bl} L{x0},{y0+tl} A{tl},{tl} 0 0,1 {x0+tl},{y0} Z'
        pattrs={A+'pathData':pdata}
        if sd['bg_rgba'] is not None:
            r,g,b,al=sd['bg_rgba']
            pattrs[A+'fillColor']=f'#{r:02x}{g:02x}{b:02x}' if al==255 else f'#{al:02x}{r:02x}{g:02x}{b:02x}'
        if sd['stroke_rgba'] is not None and stw>0:
            r,g,b,al=sd['stroke_rgba']
            pattrs[A+'strokeColor']=f'#{r:02x}{g:02x}{b:02x}' if al==255 else f'#{al:02x}{r:02x}{g:02x}{b:02x}'
            pattrs[A+'strokeWidth']=str(stw)
        ET.SubElement(vec,'path',pattrs)
        with tempfile.TemporaryDirectory() as td:
            tmp_vec=Path(td)/'shape_vec.xml'
            ET.ElementTree(vec).write(tmp_vec,encoding='utf-8',xml_declaration=True)
            if target_name not in images:images[target_name]=rasterize_vector(tmp_vec,density)
    def rasterize_frame_element(xroot, dref, target_name, seen, xml_file=None):
        if xroot.tag=='vector':
            _,_,_,_,v_am=resolve_and_rasterize_vector(xml_file if xml_file is not None else xroot,target_name)
            return (True,'',v_am)
        if xroot.tag=='bitmap':
            ba=strip_tools(xroot.attrib)
            allowed_bmp={A+'src',A+'tint',A+'tintMode',A+'gravity',A+'tileMode',A+'tileModeX',A+'tileModeY',A+'antialias',A+'dither',A+'filter',A+'mipMap',A+'alpha',A+'autoMirrored',A+'visible'}
            if len(xroot) or (set(ba)-allowed_bmp) or A+'src' not in ba:raise ValueError('Invalid bitmap drawable: '+dref)
            if A+'gravity' in ba:
                gparts=[p.strip() for p in resolve(ba[A+'gravity']).split('|') if p.strip()]
                if not gparts or any(p not in BITMAP_GRAVITY_TOKENS for p in gparts):raise ValueError('Unsupported bitmap gravity '+ba[A+'gravity'])
            if A+'tintMode' in ba and resolve(ba[A+'tintMode']).strip() not in {'src_atop','src_in','src_over','multiply','screen','add'}:
                raise ValueError('Unsupported bitmap tintMode '+ba[A+'tintMode'])
            ok,st,am=rasterize_frame_drawable(resolve(ba[A+'src']),target_name,seen+(dref,))
            if ok and A+'tint' in ba:
                r,g,b,al=static_color_rgba(ba[A+'tint'])
                indicator_tints[target_name]=f'#{al:02x}{r:02x}{g:02x}{b:02x}'
            return (ok,st,am)
        if xroot.tag in {'rotate','scale','clip'}:
            kind,payload,wrap_am=validate_drawable_wrapper(xroot,dref)
            ok,st,am=(rasterize_frame_drawable(payload,target_name,seen+(dref,)) if kind=='ref'
                else rasterize_frame_element(payload,dref,target_name,seen))
            return (ok,st,am or wrap_am)
        if xroot.tag in {'level-list','transition'}:
            kind,payload,list_am=select_list_drawable_item(xroot,dref)
            ok,st,am=(rasterize_frame_drawable(payload,target_name,seen+(dref,)) if kind=='ref'
                else rasterize_frame_element(payload,dref,target_name,seen))
            return (ok,st,am or list_am)
        if xroot.tag=='inset':
            ia=strip_tools(xroot.attrib)
            if A+'drawable' in ia and not len(xroot):
                return rasterize_frame_drawable(resolve(ia[A+'drawable']),target_name,seen+(dref,))
            if A+'drawable' not in ia and len(xroot)==1:
                return rasterize_frame_element(xroot[0],dref,target_name,seen)
            raise ValueError('Invalid inset drawable: '+dref)
        if xroot.tag=='layer-list':
            items=list(xroot)
            if not items or any(it.tag!='item' for it in items):raise ValueError('Unsupported layer-list child: '+dref)
            for it in reversed(items):
                ia=strip_tools(it.attrib)
                if A+'drawable' in ia and not len(it):
                    dr=resolve(ia[A+'drawable'])
                    if dr=='@null':continue
                    ok,st,am=rasterize_frame_drawable(dr,target_name,seen+(dref,))
                elif A+'drawable' not in ia and len(it)==1:
                    ok,st,am=rasterize_frame_element(it[0],dref,target_name,seen)
                else:
                    raise ValueError('Invalid layer-list item: '+dref)
                if ok:return (ok,st,am)
            return (False,'',False)
        if xroot.tag=='ripple':
            ra=strip_tools(xroot.attrib)
            if A+'color' not in ra:raise ValueError('Ripple drawable requires android:color: '+dref)
            static_color_rgba(ra[A+'color'])
            content_items=[c for c in xroot if c.tag=='item' and c.attrib.get(A+'id')!='@android:id/mask']
            for it in reversed(content_items):
                ia=strip_tools(it.attrib)
                if A+'drawable' in ia and not len(it):
                    dr=resolve(ia[A+'drawable'])
                    if dr=='@null':continue
                    ok,st,am=rasterize_frame_drawable(dr,target_name,seen+(dref,))
                elif A+'drawable' not in ia and len(it)==1:
                    ok,st,am=rasterize_frame_element(it[0],dref,target_name,seen)
                else:
                    raise ValueError('Invalid ripple content item: '+dref)
                if ok:return (ok,st,am)
            return (False,'',False)
        if xroot.tag=='color':
            ca=strip_tools(xroot.attrib)
            if len(xroot) or set(ca)!={A+'color'}:raise ValueError('Unsupported color drawable: '+dref)
            r,g,b,al=static_color_rgba(ca[A+'color'])
            if target_name not in images:
                px_sz=max(1,int(round(24.0*density/160.0)))
                images[target_name]=(px_sz,px_sz,bytes([r,g,b,al])*(px_sz*px_sz),160.0/density)
            return (True,'',False)
        if xroot.tag=='animated-vector':
            _,first_key,_,_,_,v_am=decode_animated_vector_frames(xroot,dref,target_name,seen,sample_frames=False)
            return (first_key in images,'',v_am)
        if xroot.tag=='animation-list':
            chosen=validate_animation_list(xroot,dref)
            cia=strip_tools(chosen.attrib)
            if A+'drawable' in cia and not len(chosen):
                return rasterize_frame_drawable(resolve(cia[A+'drawable']),target_name,seen+(dref,))
            if A+'drawable' not in cia and len(chosen)==1:
                return rasterize_frame_element(chosen[0],dref,target_name,seen)
            raise ValueError('Invalid animation-list item: '+dref)
        if xroot.tag in {'selector','animated-selector'}:
            default_it=None;first_it=None
            for it in xroot:
                if xroot.tag=='animated-selector' and it.tag=='transition':continue
                if it.tag!='item':raise ValueError('Invalid selector item: '+dref)
                ia=strip_tools(it.attrib)
                has_state=any(k.startswith(A) and k[len(A):].startswith('state_') for k in ia)
                if first_it is None:first_it=it
                if not has_state and default_it is None:default_it=it
            chosen=default_it if default_it is not None else first_it
            if chosen is None:return (False,'',False)
            cia=strip_tools(chosen.attrib)
            if A+'drawable' in cia and not len(chosen):
                return rasterize_frame_drawable(resolve(cia[A+'drawable']),target_name,seen+(dref,))
            if A+'color' in cia and not len(chosen):
                r,g,b,al=static_color_rgba(cia[A+'color'])
                if A+'alpha' in cia:al=int(al*float(resolve(cia[A+'alpha']))+0.5)
                if target_name not in images:
                    px_sz=max(1,int(round(24.0*density/160.0)))
                    images[target_name]=(px_sz,px_sz,bytes([r,g,b,al])*(px_sz*px_sz),160.0/density)
                return (True,'',False)
            if len(chosen)==1:
                return rasterize_frame_element(chosen[0],dref,target_name,seen)
            return (False,'',False)
        if xroot.tag=='shape':
            rasterize_shape_to_image(xroot,dref,target_name)
            return (True,'',False)
        return (False,'',False)
    def rasterize_frame_drawable(dref, target_name, seen=()):
        if dref in seen:raise ValueError('Cyclic drawable reference '+dref)
        if dref=='@null':return (False,'',False)
        if not dref.startswith('@drawable/'):
            r,g,b,al=static_color_rgba(dref)
            if target_name not in images:
                px_sz=max(1,int(round(24.0*density/160.0)))
                images[target_name]=(px_sz,px_sz,bytes([r,g,b,al])*(px_sz*px_sz),160.0/density)
            return (True,'',False)
        if not resource_roots:raise ValueError('Drawable needs --resources: '+dref)
        dname=dref.split('/')[-1]
        candidates,nodpi,xml_file=find_drawable_candidates(dname)
        if len({dpi for dpi,_ in candidates})!=len(candidates):raise ValueError('Ambiguous PNG drawable density '+dref)
        image_file=None;source_density=160
        if candidates:
            higher=sorted((dpi,file) for dpi,file in candidates if dpi>=density)
            source_density,image_file=higher[0] if higher else max(candidates)
        elif nodpi is not None:
            image_file=nodpi
        if image_file is not None:
            w,h,pixels,scale,_,stretch_spec=decode_drawable_image(image_file,source_density)
            if target_name not in images:
                images[target_name]=(w,h,pixels,scale)
            return (True,stretch_spec,False)
        if xml_file is not None:
            return rasterize_frame_element(ET.parse(xml_file).getroot(),dref,target_name,seen,xml_file=xml_file)
        return (False,'',False)
    def decode_animation_list_frames(root, ref_label, base_key, seen=()):
        validate_animation_list(root,ref_label)
        oneshot='1' if resolve(root.attrib.get(A+'oneshot','false')).strip()=='true' else '0'
        frames=[]
        first_key='';first_st='';any_am=False
        for idx,it in enumerate(list(root)):
            ia=strip_tools(it.attrib)
            dur=resolve(ia[A+'duration']).strip() if A+'duration' in ia else '100'
            if A+'drawable' in ia and not len(it):
                dr=resolve(ia[A+'drawable'])
                if dr=='@null':raise ValueError('Invalid animation-list item: '+ref_label)
                frame_key=dr.split('/')[-1] if dr.startswith('@drawable/') else f'{base_key}__f{idx}'
                f_ok,f_st,f_am=rasterize_frame_drawable(dr,frame_key,seen+(ref_label,))
                if not f_ok:raise ValueError('Unresolved PNG drawable '+dr)
            elif A+'drawable' not in ia and len(it)==1:
                frame_key=f'{base_key}__f{idx}'
                f_ok,f_st,f_am=rasterize_frame_element(it[0],ref_label,frame_key,seen+(ref_label,))
                if not f_ok:raise ValueError('Invalid animation-list item: '+ref_label)
            else:
                raise ValueError('Invalid animation-list item: '+ref_label)
            if idx==0:
                first_key=frame_key;first_st=f_st
            any_am=any_am or f_am
            frames.append(f'{frame_key}:{dur}')
        return f'{oneshot}|'+','.join(frames),first_key,first_st,any_am
    def decode_animated_selector_transitions(root, ref_label, base_key, item_id_to_mask, seen=()):
        transitions=[]
        any_am=False
        seen_next=seen if ref_label in seen else seen+(ref_label,)
        tr_idx=0
        for tr in root:
            if tr.tag!='transition':continue
            ta=strip_tools(tr.attrib)
            allowed_tr={A+'fromId',A+'toId',A+'reversible',A+'drawable'}
            if set(ta)-allowed_tr:raise ValueError('Unsupported animated-selector transition attribute: '+ref_label)
            if A+'fromId' not in ta or A+'toId' not in ta:raise ValueError('Invalid animated-selector transition: '+ref_label)
            from_raw=resolve(ta[A+'fromId']).strip()
            to_raw=resolve(ta[A+'toId']).strip()
            from_id=id_key(from_raw)
            to_id=id_key(to_raw)
            if from_id is None or from_id not in item_id_to_mask:raise ValueError('Invalid animated-selector transition fromId '+from_raw)
            if to_id is None or to_id not in item_id_to_mask:raise ValueError('Invalid animated-selector transition toId '+to_raw)
            rev='0'
            if A+'reversible' in ta:
                rv=resolve(ta[A+'reversible']).strip()
                if rv not in {'true','false'}:raise ValueError('Invalid animated-selector transition reversible '+rv)
                rev='1' if rv=='true' else '0'
            from_mask=item_id_to_mask[from_id]
            to_mask=item_id_to_mask[to_id]
            if A+'drawable' in ta and not len(tr):
                dr=resolve(ta[A+'drawable']).strip()
                if dr in seen_next:raise ValueError('Cyclic drawable reference '+dr)
                if not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',dr):raise ValueError('Unsupported animated-selector transition drawable '+dr)
                if not resource_roots:raise ValueError('Drawable needs --resources: '+dr)
                tr_dname=dr.split('/')[-1]
                tr_xml=find_drawable_xml(tr_dname)
                if tr_xml is None:raise ValueError('Unresolved animated-selector transition drawable '+dr)
                tr_root=ET.parse(tr_xml).getroot()
                if tr_root.tag!='animation-list':raise ValueError('Unsupported animated-selector transition drawable '+dr)
                anim_spec,_,_,tr_am=decode_animation_list_frames(tr_root,dr,tr_dname,seen_next)
            elif A+'drawable' not in ta and len(tr)==1:
                if tr[0].tag!='animation-list':raise ValueError('Unsupported animated-selector transition child '+tr[0].tag)
                anim_spec,_,_,tr_am=decode_animation_list_frames(tr[0],ref_label,f'{base_key}__t{tr_idx}',seen_next)
            else:
                raise ValueError('Invalid animated-selector transition: '+ref_label)
            frames_part=anim_spec.split('|',1)[1]
            transitions.append(f'{from_mask}:{to_mask}:{rev}|{frames_part}')
            any_am=any_am or tr_am
            tr_idx+=1
        return transitions,any_am
    def eval_avd_interpolator(kind, u):
        if u<=0.0:return 0.0
        if u>=1.0:return 1.0
        if kind=='linear':return u
        if kind=='accelerate':return u*u
        if kind=='decelerate':return 1.0-(1.0-u)*(1.0-u)
        if kind=='fast_out_slow_in':
            lo,hi=0.0,1.0
            for _ in range(16):
                mid=0.5*(lo+hi)
                mt=1.0-mid
                x=3.0*mt*mt*mid*0.4+3.0*mt*mid*mid*0.2+mid*mid*mid
                if x<u:lo=mid
                else:hi=mid
            t=0.5*(lo+hi)
            mt=1.0-t
            return 3.0*mt*t*t+t*t*t
        if kind=='overshoot':
            t=u-1.0
            return t*t*(3.0*t+2.0)+1.0
        if kind=='anticipate':
            return u*u*(3.0*u-2.0)
        return 0.5*(1.0-math.cos(math.pi*u))
    def decode_animated_vector_frames(root, ref_label, base_key, seen=(), sample_frames=True):
        avd_group_props={'rotation','scaleX','scaleY','translateX','translateY'}
        avd_path_props={'trimPathStart','trimPathEnd','trimPathOffset','fillAlpha','strokeAlpha'}
        avd_clamp_props={'alpha','fillAlpha','strokeAlpha','trimPathStart','trimPathEnd'}
        avd_all_props=avd_group_props | avd_path_props | {'alpha'}
        avd_defaults={'rotation':0.0,'scaleX':1.0,'scaleY':1.0,'translateX':0.0,'translateY':0.0,'trimPathStart':0.0,'trimPathEnd':1.0,'trimPathOffset':0.0,'fillAlpha':1.0,'strokeAlpha':1.0,'alpha':1.0}
        ava=strip_tools(root.attrib)
        if set(ava)-{A+'drawable'}:raise ValueError('Unsupported animated-vector attribute: '+ref_label)
        seen_next=seen if ref_label in seen else seen+(ref_label,)
        base_vec=None
        if A+'drawable' in ava:
            dr=resolve(ava[A+'drawable']).strip()
            if dr in seen_next:raise ValueError('Cyclic drawable reference '+dr)
            if not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',dr):raise ValueError('Unsupported animated-vector drawable '+dr)
            if not resource_roots:raise ValueError('Drawable needs --resources: '+dr)
            vx=find_drawable_xml(dr.split('/')[-1])
            if vx is None:raise ValueError('Unresolved animated-vector drawable '+dr)
            base_vec=ET.parse(vx).getroot()
            if base_vec.tag!='vector':raise ValueError('Animated-vector drawable must be a vector: '+dr)
        targets=[]
        for child in root:
            if child.tag==AAPT+'attr':
                ca=strip_tools(child.attrib)
                if set(ca)!={'name'} or ca.get('name')!='android:drawable' or len(child)!=1 or child[0].tag!='vector':
                    raise ValueError('Unsupported animated-vector aapt:attr: '+ref_label)
                if base_vec is not None:raise ValueError('Duplicate animated-vector drawable: '+ref_label)
                base_vec=child[0]
            elif child.tag=='target':
                targets.append(child)
            else:
                raise ValueError('Unsupported animated-vector child '+child.tag)
        if base_vec is None:raise ValueError('Animated-vector requires drawable: '+ref_label)
        named_tags={}
        named_init_vals={}
        for el in base_vec.iter():
            if A+'name' in el.attrib:
                ename=resolve(el.attrib[A+'name']).strip()
                if ename:
                    named_tags.setdefault(ename,set()).add(el.tag)
                    for pk in avd_all_props:
                        if A+pk in el.attrib and (ename,pk) not in named_init_vals:
                            try: named_init_vals[(ename,pk)]=float(resolve(el.attrib[A+pk]).strip())
                            except ValueError: pass
        tracks=[]
        repeat_flag=0
        def parse_avd_float(raw, err_ref):
            s=resolve(raw).strip()
            if not re.fullmatch(r'-?(?:\d+(?:\.\d*)?|\.\d+)',s):raise ValueError('Invalid animator value '+err_ref)
            v=float(s)
            if not math.isfinite(v):raise ValueError('Invalid animator value '+err_ref)
            return v
        def parse_avd_anim(ael, target_name, base_offset_ms, anim_ref_label):
            nonlocal repeat_flag
            if ael.tag=='set':
                sa=strip_tools(ael.attrib)
                if set(sa)-{A+'ordering'}:raise ValueError('Unsupported AnimatorSet attribute: '+anim_ref_label)
                ordering=resolve(sa.get(A+'ordering','together')).strip()
                if ordering not in {'together','sequentially'}:raise ValueError('Unsupported AnimatorSet ordering: '+ordering)
                if not len(ael):raise ValueError('Empty AnimatorSet: '+anim_ref_label)
                if ordering=='together':
                    max_dur=0
                    for ch in ael:
                        cd=parse_avd_anim(ch,target_name,base_offset_ms,anim_ref_label)
                        if cd>max_dur:max_dur=cd
                    return max_dur
                cursor=0
                for ch in ael:
                    cursor+=parse_avd_anim(ch,target_name,base_offset_ms+cursor,anim_ref_label)
                return cursor
            if ael.tag!='objectAnimator':raise ValueError('Unsupported animated-vector animator '+ael.tag)
            ra=strip_tools(ael.attrib)
            allowed={A+'propertyName',A+'valueFrom',A+'valueTo',A+'duration',A+'startOffset',A+'interpolator',A+'valueType',A+'repeatCount',A+'repeatMode'}
            if len(ael) or set(ra)-allowed:raise ValueError('Unsupported animated-vector objectAnimator: '+anim_ref_label)
            prop=resolve(ra.get(A+'propertyName','')).strip()
            if prop not in avd_all_props:raise ValueError('Unsupported animated-vector property '+prop)
            ttags=named_tags[target_name]
            if prop in avd_group_props and 'group' not in ttags:
                raise ValueError('Incompatible animated-vector property '+prop+' for target '+target_name)
            if prop in avd_path_props and 'path' not in ttags:
                raise ValueError('Incompatible animated-vector property '+prop+' for target '+target_name)
            if prop=='alpha' and not (ttags & {'vector','group','path'}):
                raise ValueError('Incompatible animated-vector property alpha for target '+target_name)
            vt=resolve(ra.get(A+'valueType','floatType')).strip()
            if vt!='floatType':raise ValueError('Unsupported animated-vector valueType '+vt)
            try:
                duration=int(resolve(ra.get(A+'duration','300')).strip())
                offset=int(resolve(ra.get(A+'startOffset','0')).strip())
            except ValueError:raise ValueError('Invalid animator duration '+anim_ref_label)
            if not 0<=duration<=60000 or not 0<=offset<=60000 or base_offset_ms+offset>60000:
                raise ValueError('Invalid animator duration or startOffset '+anim_ref_label)
            if A+'repeatCount' in ra:
                rc=resolve(ra[A+'repeatCount']).strip()
                if rc!='infinite' and not re.fullmatch(r'-1|\d+',rc):raise ValueError('Invalid animator repeatCount '+rc)
                if rc=='infinite' or int(rc)==-1 or int(rc)>0:repeat_flag=1
            if A+'repeatMode' in ra:
                rm=resolve(ra[A+'repeatMode']).strip()
                if rm not in {'restart','reverse'}:raise ValueError('Invalid animator repeatMode '+rm)
            curve=ra.get(A+'interpolator','@android:anim/accelerate_decelerate_interpolator')
            interp=parse_interpolator_kind(curve,'Unsupported animator interpolator ')
            if A+'valueTo' not in ra:raise ValueError('Invalid animator value '+anim_ref_label)
            v_to=parse_avd_float(ra[A+'valueTo'],anim_ref_label)
            v_from=parse_avd_float(ra[A+'valueFrom'],anim_ref_label) if A+'valueFrom' in ra else named_init_vals.get((target_name,prop),avd_defaults[prop])
            if prop in avd_clamp_props and not (0.0<=v_from<=1.0 and 0.0<=v_to<=1.0):
                raise ValueError('Invalid animated-vector '+prop+' value in '+anim_ref_label)
            tracks.append((target_name,prop,base_offset_ms+offset,duration,v_from,v_to,interp))
            return offset+duration
        for target in targets:
            ta=strip_tools(target.attrib)
            if set(ta)-{A+'name',A+'animation'}:raise ValueError('Unsupported animated-vector target attribute: '+ref_label)
            tname=resolve(ta.get(A+'name','')).strip()
            if not tname:raise ValueError('Animated-vector target requires android:name: '+ref_label)
            if tname not in named_tags:raise ValueError('Unresolved animated-vector target '+tname)
            if A+'animation' in ta and not len(target):
                aref=resolve(ta[A+'animation']).strip()
                if not re.fullmatch(r'@animator/[a-z][a-z0-9_]*',aref) or not resource_roots:
                    raise ValueError('Unsupported animated-vector animation '+aref)
                apath=find_resource_file('animator',aref.split('/')[-1]+'.xml')
                if apath is None:raise ValueError('Unresolved animator '+aref)
                parse_avd_anim(ET.parse(apath).getroot(),tname,0,aref)
            elif A+'animation' not in ta and len(target)==1:
                aattr=target[0]
                aca=strip_tools(aattr.attrib)
                if aattr.tag!=AAPT+'attr' or set(aca)!={'name'} or aca.get('name')!='android:animation' or len(aattr)!=1:
                    raise ValueError('Invalid animated-vector target: '+ref_label)
                parse_avd_anim(aattr[0],tname,0,ref_label)
            else:
                raise ValueError('Invalid animated-vector target: '+ref_label)
        def clone_vec(el):
            c=ET.Element(el.tag,{k:val for k,val in el.attrib.items() if not k.startswith(TOOLS)})
            for ch in el:c.append(clone_vec(ch))
            return c
        def fmt_avd_num(v):
            if abs(v)<1e-7:return '0'
            return f'{v:.6f}'.rstrip('0').rstrip('.') or '0'
        def build_frame_vector(time_ms):
            fv=clone_vec(base_vec)
            updates={}
            for tname,prop,start_ms,dur_ms,v_from,v_to,interp in tracks:
                if time_ms<start_ms:continue
                u=1.0 if dur_ms==0 else min(1.0,max(0.0,(time_ms-start_ms)/dur_ms))
                val=v_from+(v_to-v_from)*eval_avd_interpolator(interp,u)
                if prop in avd_clamp_props:val=min(1.0,max(0.0,val))
                updates[(tname,prop)]=val
            alpha_targets=[]
            for el in fv.iter():
                if A+'name' not in el.attrib:continue
                ename=resolve(el.attrib[A+'name']).strip()
                if not ename:continue
                if el.tag=='vector':
                    if (ename,'alpha') in updates:el.attrib[A+'alpha']=fmt_avd_num(updates[(ename,'alpha')])
                elif el.tag=='group':
                    for pk in avd_group_props:
                        if (ename,pk) in updates:el.attrib[A+pk]=fmt_avd_num(updates[(ename,pk)])
                    if (ename,'alpha') in updates:alpha_targets.append((el,updates[(ename,'alpha')]))
                elif el.tag=='path':
                    for pk in avd_path_props:
                        if (ename,pk) in updates:el.attrib[A+pk]=fmt_avd_num(updates[(ename,pk)])
                    if (ename,'alpha') in updates:alpha_targets.append((el,updates[(ename,'alpha')]))
            for ael,aval in alpha_targets:
                paths=[ael] if ael.tag=='path' else [p for p in ael.iter() if p.tag=='path']
                for p in paths:
                    for ak in ('fillAlpha','strokeAlpha'):
                        cur=float(resolve(p.attrib.get(A+ak,'1')).strip())
                        p.attrib[A+ak]=fmt_avd_num(min(1.0,max(0.0,cur*aval)))
            fv.attrib.pop(A+'name',None)
            return fv
        if not sample_frames or not tracks:
            fv0=build_frame_vector(0.0)
            vw,vh,vtint,_,auto_m=resolve_and_rasterize_vector(fv0,base_key)
            return None,base_key,vw,vh,vtint,auto_m
        total_dur_ms=max(start_ms+dur_ms for _,_,start_ms,dur_ms,_,_,_ in tracks)
        eff_dur_ms=max(1,total_dur_ms)
        frame_keys=[]
        vw=vh=24.0;vtint=None;auto_m=False
        for idx in range(8):
            t_ms=(idx/7.0)*total_dur_ms
            fv=build_frame_vector(t_ms)
            fk=f'{base_key}__avd{idx}'
            fw,fh,ft,_,fam=resolve_and_rasterize_vector(fv,fk)
            if idx==0:vw,vh,vtint,auto_m=fw,fh,ft,fam
            frame_keys.append(fk)
        avd_spec=f'{repeat_flag}:{eff_dur_ms}|'+','.join(frame_keys)
        return avd_spec,frame_keys[0],vw,vh,vtint,auto_m
    def map_dynamic_gravity(raw_grav, default_g='start', prefer_vertical=False):
        if raw_grav is None:return default_g
        gp=[p.strip() for p in resolve(raw_grav).split('|') if p.strip()]
        if not gp:return default_g
        if prefer_vertical:
            if 'top' in gp:return 'top'
            if 'bottom' in gp:return 'bottom'
            if 'start' in gp or 'left' in gp:return 'start'
            if 'end' in gp or 'right' in gp:return 'end'
            return 'center'
        if 'start' in gp or 'left' in gp:return 'start'
        if 'end' in gp or 'right' in gp:return 'end'
        if 'top' in gp:return 'top'
        if 'bottom' in gp:return 'bottom'
        return 'center'
    def parse_wrap_pivot_frac(raw):
        pv=resolve(raw).strip()
        if pv.endswith('%p') or pv.endswith('%P'):val=float(pv[:-2])/100.0
        elif pv.endswith('%'):val=float(pv[:-1])/100.0
        elif pv.endswith(('dp','dip','px')):
            num_s=pv[:-3] if pv.endswith('dip') else pv[:-2]
            val=0.0 if float(num_s)==0.0 else 0.5
        else:val=float(pv)
        return max(0.0,min(1.0,val))
    def parse_scale_frac(raw):
        sv=resolve(raw).strip()
        if sv.endswith('%p'):val=float(sv[:-2])/100.0
        elif sv.endswith('%'):val=float(sv[:-1])/100.0
        else:val=float(sv)
        return max(0.0,min(1.0,val))
    def bundle_dynamic_item(kind, payload, ref_label, fallback_key, seen):
        if kind=='ref':
            dr=resolve(payload)
            if dr=='@null':return ''
            item_key=dr.split('/')[-1] if dr.startswith('@drawable/') else fallback_key
            ok,_,_=rasterize_frame_drawable(dr,item_key,seen+(ref_label,))
            if not ok:raise ValueError('Unresolved drawable '+dr)
            return item_key
        ok,_,_=rasterize_frame_element(payload,ref_label,fallback_key,seen+(ref_label,))
        if not ok:raise ValueError('Invalid dynamic drawable item: '+ref_label)
        return fallback_key
    def decode_dynamic_drawable(root, ref_label, base_key, seen=()):
        if root.tag=='level-list':
            select_list_drawable_item(root,ref_label)
            entries=[]
            for idx,it in enumerate(list(root)):
                ia=strip_tools(it.attrib)
                lo=max(0,int(resolve(ia[A+'minLevel']).strip())) if A+'minLevel' in ia else 0
                hi=max(0,int(resolve(ia[A+'maxLevel']).strip())) if A+'maxLevel' in ia else 0
                if A+'drawable' in ia and not len(it):
                    k=bundle_dynamic_item('ref',ia[A+'drawable'],ref_label,f'{base_key}__f{idx}',seen)
                else:
                    k=bundle_dynamic_item('child',it[0],ref_label,f'{base_key}__f{idx}',seen)
                entries.append(f'{lo}:{hi}:{k}')
            return 'level|'+','.join(entries)
        if root.tag=='transition':
            select_list_drawable_item(root,ref_label)
            items=list(root)
            keys=[]
            for idx,it in enumerate(items[:2]):
                ia=strip_tools(it.attrib)
                if A+'drawable' in ia and not len(it):
                    k=bundle_dynamic_item('ref',ia[A+'drawable'],ref_label,f'{base_key}__f{idx}',seen)
                else:
                    k=bundle_dynamic_item('child',it[0],ref_label,f'{base_key}__f{idx}',seen)
                keys.append(k)
            if len(keys)==1:keys.append(keys[0])
            return f'transition|250|{keys[0]},{keys[1]}'
        kind,payload,_=validate_drawable_wrapper(root,ref_label)
        wa=strip_tools(root.attrib)
        k=bundle_dynamic_item(kind,payload,ref_label,base_key,seen)
        if root.tag=='clip':
            orient='v' if (A+'clipOrientation' in wa and resolve(wa[A+'clipOrientation']).strip()=='vertical') else 'h'
            grav=map_dynamic_gravity(wa.get(A+'gravity'),default_g=('top' if orient=='v' else 'start'),prefer_vertical=(orient=='v'))
            return f'clip|{orient}|{grav}|{k}'
        if root.tag=='rotate':
            fd=float(resolve(wa[A+'fromDegrees']).strip()) if A+'fromDegrees' in wa else 0.0
            td=float(resolve(wa[A+'toDegrees']).strip()) if A+'toDegrees' in wa else 360.0
            px=parse_wrap_pivot_frac(wa[A+'pivotX']) if A+'pivotX' in wa else 0.5
            py=parse_wrap_pivot_frac(wa[A+'pivotY']) if A+'pivotY' in wa else 0.5
            return f'rotate|{fmt_num(fd)},{fmt_num(td)}|{fmt_num(px)},{fmt_num(py)}|{k}'
        if root.tag=='scale':
            sw=parse_scale_frac(wa[A+'scaleWidth']) if A+'scaleWidth' in wa else 1.0
            sh=parse_scale_frac(wa[A+'scaleHeight']) if A+'scaleHeight' in wa else 1.0
            grav=map_dynamic_gravity(wa.get(A+'scaleGravity'),default_g='start',prefer_vertical=False)
            return f'scale|{fmt_num(sw)},{fmt_num(sh)}|{grav}|{k}'
        raise ValueError('Unsupported dynamic drawable: '+ref_label)
    def parse_shape_element(root, ref_label):
        shape_attrs=strip_tools(root.attrib)
        allowed_root={A+'shape',A+'dither',A+'visible',A+'useLevel',A+'tint',A+'tintMode',A+'innerRadius',A+'innerRadiusRatio',A+'thickness',A+'thicknessRatio'}
        if root.tag!='shape' or (set(shape_attrs)-allowed_root) or any(c.tag not in {'solid','corners','stroke','padding','size','gradient'} for c in root):
            raise ValueError('Only solid shape drawables supported: '+ref_label)
        shape=resolve(shape_attrs.get(A+'shape','rectangle'))
        if shape not in {'rectangle','oval','line','ring'}:raise ValueError('Unsupported shape '+shape)
        solid=root.find('solid')
        solid_rgba=None
        if solid is not None:
            s_attrs=strip_tools(solid.attrib)
            if len(solid) or set(s_attrs)!={A+'color'}:raise ValueError('Unsupported shape solid: '+ref_label)
            solid_rgba=static_color_rgba(s_attrs[A+'color'])
        corners=root.find('corners')
        corner_radii=None
        if shape in {'oval','ring'}:
            radius='-1.0'
        elif shape=='line':
            radius='0.0'
        elif corners is not None:
            c_attrs=strip_tools(corners.attrib)
            allowed_corners={A+'radius',A+'topLeftRadius',A+'topRightRadius',A+'bottomRightRadius',A+'bottomLeftRadius'}
            if len(corners) or (set(c_attrs)-allowed_corners):raise ValueError('Unsupported shape corners: '+ref_label)
            base_r=float(number(c_attrs.get(A+'radius','0dp')))
            tl=float(number(c_attrs[A+'topLeftRadius'])) if A+'topLeftRadius' in c_attrs else base_r
            tr=float(number(c_attrs[A+'topRightRadius'])) if A+'topRightRadius' in c_attrs else base_r
            br=float(number(c_attrs[A+'bottomRightRadius'])) if A+'bottomRightRadius' in c_attrs else base_r
            bl=float(number(c_attrs[A+'bottomLeftRadius'])) if A+'bottomLeftRadius' in c_attrs else base_r
            if tl==tr==br==bl:
                radius=str(tl)
            else:
                radius='0.0'
                corner_radii=f'{tl},{tr},{br},{bl}'
        else:
            radius='0.0'
        stroke=root.find('stroke')
        stroke_rgba=None;stroke_width='0.0'
        if stroke is not None:
            st_attrs=strip_tools(stroke.attrib)
            allowed_stroke={A+'width',A+'color',A+'dashWidth',A+'dashGap'}
            if len(stroke) or not {A+'width',A+'color'} <= set(st_attrs) or (set(st_attrs)-allowed_stroke):raise ValueError('Only solid strokes supported: '+ref_label)
            stroke_rgba=static_color_rgba(st_attrs[A+'color'])
            stroke_width=number(st_attrs[A+'width'])
            for dk in (A+'dashWidth',A+'dashGap'):
                if dk in st_attrs:number(st_attrs[dk])
        grad=root.find('gradient')
        gradient_str=None
        if grad is not None:
            g_attrs=strip_tools(grad.attrib)
            allowed_grad={A+'type',A+'angle',A+'startColor',A+'centerColor',A+'endColor',A+'centerX',A+'centerY',A+'dither',A+'useLevel',A+'gradientRadius'}
            if len(grad) or (set(g_attrs)-allowed_grad):raise ValueError('Unsupported shape gradient: '+ref_label)
            gtype=resolve(g_attrs.get(A+'type','linear'))
            if gtype not in {'linear','radial','sweep'}:raise ValueError('Unsupported gradient type '+gtype)
            grad_radius_dp=0.0
            if A+'gradientRadius' in g_attrs:
                gr=resolve(g_attrs[A+'gradientRadius']).strip()
                gr_tok=theme_token(gr)
                if gr_tok is not None and gr_tok in THEME_DIMENS:gr=THEME_DIMENS[gr_tok]
                gr_m=re.fullmatch(r'(\d+(?:\.\d+)?|\.\d+)(dp|dip|sp|px|%p|%)?',gr)
                if not gr_m:raise ValueError('Invalid gradientRadius '+gr)
                gr_num=float(gr_m.group(1));gr_unit=gr_m.group(2)
                if gr_unit in {'%','%p'}:
                    grad_radius_dp=0.0
                elif gr_unit=='px':
                    grad_radius_dp=round(gr_num*160.0/density,4)
                else:
                    grad_radius_dp=gr_num
            raw_angle=resolve(g_attrs.get(A+'angle','0'))
            try: angle=float(raw_angle)
            except ValueError: raise ValueError('Invalid gradient angle '+raw_angle)
            if angle!=angle or angle in (float('inf'),float('-inf')):raise ValueError('Invalid gradient angle '+raw_angle)
            start_rgba=static_color_rgba(g_attrs[A+'startColor']) if A+'startColor' in g_attrs else (0,0,0,0)
            end_rgba=static_color_rgba(g_attrs[A+'endColor']) if A+'endColor' in g_attrs else (0,0,0,0)
            if gtype=='radial':
                cx=parse_fraction(g_attrs[A+'centerX']) if A+'centerX' in g_attrs else 0.5
                cy=parse_fraction(g_attrs[A+'centerY']) if A+'centerY' in g_attrs else 0.5
                center_str=rgba_to_csv(static_color_rgba(g_attrs[A+'centerColor'])) if A+'centerColor' in g_attrs else ''
                r_str='0' if grad_radius_dp==0.0 else str(grad_radius_dp)
                gradient_str=f'radial|{cx},{cy},{r_str}|{rgba_to_csv(start_rgba)}|{center_str}|{rgba_to_csv(end_rgba)}'
            elif gtype=='sweep':
                cx=parse_fraction(g_attrs[A+'centerX']) if A+'centerX' in g_attrs else 0.5
                cy=parse_fraction(g_attrs[A+'centerY']) if A+'centerY' in g_attrs else 0.5
                center_str=rgba_to_csv(static_color_rgba(g_attrs[A+'centerColor'])) if A+'centerColor' in g_attrs else ''
                gradient_str=f'sweep|{cx},{cy}|{rgba_to_csv(start_rgba)}|{center_str}|{rgba_to_csv(end_rgba)}'
            elif A+'centerColor' in g_attrs:
                center_rgba=static_color_rgba(g_attrs[A+'centerColor'])
                cx=parse_fraction(g_attrs[A+'centerX']) if A+'centerX' in g_attrs else None
                cy=parse_fraction(g_attrs[A+'centerY']) if A+'centerY' in g_attrs else None
                if cx is None and cy is None:
                    gradient_str=f'{angle},{rgba_to_csv(start_rgba)},{rgba_to_csv(center_rgba)},{rgba_to_csv(end_rgba)}'
                else:
                    amod=angle%180.0
                    if abs(amod-90.0)<1e-3:pos=cy if cy is not None else cx
                    elif abs(amod)<1e-3:pos=cx if cx is not None else cy
                    else:pos=(cx+cy)*0.5 if (cx is not None and cy is not None) else (cx if cx is not None else cy)
                    gradient_str=f'{angle},{rgba_to_csv(start_rgba)},{rgba_to_csv(center_rgba)},{rgba_to_csv(end_rgba)},{pos}'
            else:
                gradient_str=f'{angle},{rgba_to_csv(start_rgba)},{rgba_to_csv(end_rgba)}'
        padding=root.find('padding')
        padding_vals=None
        if padding is not None:
            p_attrs=strip_tools(padding.attrib)
            if len(padding) or set(p_attrs)-{A+'left',A+'top',A+'right',A+'bottom'}:raise ValueError('Unsupported shape padding: '+ref_label)
            padding_vals=[float(number(p_attrs.get(A+side,'0dp'))) for side in ('left','top','right','bottom')]
        shape_size=root.find('size')
        size_vals=[0.0,0.0]
        if shape_size is not None:
            sz_attrs=strip_tools(shape_size.attrib)
            if len(shape_size) or set(sz_attrs)-{A+'width',A+'height'}:raise ValueError('Unsupported shape size: '+ref_label)
            size_vals=[float(number(sz_attrs.get(A+axis,'0dp'))) for axis in ('width','height')]
        if shape=='line':
            solid_rgba=None
            if stroke_rgba is not None:
                sw_f=float(stroke_width)
                size_vals[1]=max(size_vals[1],sw_f if sw_f>0.0 else 1.0)
        elif shape=='ring':
            if stroke_rgba is None and solid_rgba is not None:
                stroke_rgba=solid_rgba
                if A+'thickness' in shape_attrs:
                    stroke_width=number(shape_attrs[A+'thickness'])
                elif A+'thicknessRatio' in shape_attrs:
                    tr_raw=resolve(shape_attrs[A+'thicknessRatio'])
                    try: tr_v=float(tr_raw)
                    except ValueError: raise ValueError('Invalid ring thicknessRatio '+tr_raw)
                    if not tr_v>0:raise ValueError('Invalid ring thicknessRatio '+tr_raw)
                    ref_dim=min(x for x in size_vals if x>0.0) if any(x>0.0 for x in size_vals) else 36.0
                    stroke_width=str(round(ref_dim/tr_v,4))
                else:
                    stroke_width='4.0'
            solid_rgba=None
        d=empty_drawable_dict()
        d['bg']=rgba_to_rust(solid_rgba) if solid_rgba is not None else 'None'
        d['bg_rgba']=solid_rgba
        d['radius']=radius
        d['corner_radii']=corner_radii
        d['gradient']=gradient_str
        d['stroke']=rgba_to_rust(stroke_rgba) if stroke_rgba is not None else 'None'
        d['stroke_rgba']=stroke_rgba
        d['stroke_width']=stroke_width
        d['padding']=padding_vals
        d['size']=size_vals
        d['shape_kind']=shape
        return d
    def drawable_to_layers(d):
        if d['layers']:return list(d['layers'])
        if d['bg_rgba'] is not None or d['stroke_rgba'] is not None:
            rad_str=d['corner_radii'] if d['corner_radii'] is not None else d['radius']
            stroke_t=(float(d['stroke_width']),d['stroke_rgba']) if d['stroke_rgba'] is not None else None
            return [(0.0,0.0,0.0,0.0,rad_str,d['bg_rgba'],stroke_t)]
        if d['bg_selector']:
            chosen=next((r for r in d['bg_selector'] if r.startswith('0,0,')),d['bg_selector'][0])
            parts=chosen.split(',')
            rgba=tuple(int(x) for x in parts[2:6])
            if rgba[3]>0:
                return [(0.0,0.0,0.0,0.0,d['radius'],rgba,None)]
        return []
    def format_layer_rule(layer):
        l,t,r,b,rad_str,fill_rgba,stroke_t=layer
        fill_s=rgba_to_csv(fill_rgba) if fill_rgba is not None else 'none'
        base=f'{l},{t},{r},{b}|{rad_str}|{fill_s}'
        if stroke_t is not None:
            sw,sc=stroke_t
            base+=f'|{sw},{rgba_to_csv(sc)}'
        return base
    def merge_padding(a_pad,b_pad):
        if a_pad is None:return list(b_pad) if b_pad is not None else None
        if b_pad is None:return list(a_pad)
        return [max(x,y) for x,y in zip(a_pad,b_pad)]
    def resolve_drawable_ref(ref, stack=(), allow_dynamic=False):
        rv=resolve(ref)
        if rv=='@null':return empty_drawable_dict()
        if not rv.startswith('@drawable/'):
            rgba=static_color_rgba(rv)
            d=empty_drawable_dict();d['bg']=rgba_to_rust(rgba);d['bg_rgba']=rgba
            return d
        if rv in stack:raise ValueError('Cyclic drawable reference: '+' -> '.join(stack+(rv,)))
        if not resource_roots:raise ValueError('Drawable needs --resources: '+rv)
        dname=rv.split('/')[-1]
        cands,nodpi,file=find_drawable_candidates(dname)
        if file is None:
            if cands or nodpi is not None:
                if len({dpi for dpi,_ in cands})!=len(cands):raise ValueError('Ambiguous PNG drawable density '+rv)
                if cands:
                    higher=sorted((dpi,f) for dpi,f in cands if dpi>=density)
                    sdpi,img_file=higher[0] if higher else max(cands)
                else:
                    sdpi,img_file=160,nodpi
                w,h,pixels,scale,pad_dp,stretch_spec=decode_drawable_image(img_file,sdpi)
                if dname not in images:images[dname]=(w,h,pixels,scale)
                out=empty_drawable_dict()
                out['padding']=pad_dp
                out['size']=[round(w*scale,4),round(h*scale,4)]
                out['bg_image']=(dname,stretch_spec)
                return out
            raise ValueError('Unresolved drawable '+rv)
        root=ET.parse(file).getroot()
        return parse_drawable_element(root,rv,stack+(rv,),dname=dname,src_path=file,allow_dynamic=allow_dynamic)
    def flatten_layer_items(items, ref_label, stack, dname=None, allow_dynamic=False):
        out=empty_drawable_dict()
        child_allow_dyn=allow_dynamic and len(items)==1
        for idx,item in enumerate(items):
            if item.tag!='item':raise ValueError('Unsupported layer-list child: '+ref_label)
            ia=strip_tools(item.attrib)
            il=float(margin_number(ia[A+'left'])) if A+'left' in ia else float(margin_number(ia[A+'start'])) if A+'start' in ia else 0.0
            ir=float(margin_number(ia[A+'right'])) if A+'right' in ia else float(margin_number(ia[A+'end'])) if A+'end' in ia else 0.0
            it=float(margin_number(ia[A+'top'])) if A+'top' in ia else 0.0
            ib=float(margin_number(ia[A+'bottom'])) if A+'bottom' in ia else 0.0
            if A+'drawable' in ia and not len(item):
                child_d=resolve_drawable_ref(ia[A+'drawable'],stack,allow_dynamic=child_allow_dyn)
            elif A+'drawable' not in ia and len(item)==1:
                inline_key=f'{dname}__l{idx}' if dname else None
                child_d=parse_drawable_element(item[0],ref_label,stack,dname=inline_key,allow_dynamic=child_allow_dyn)
            else:
                raise ValueError('Invalid layer-list item: '+ref_label)
            for l,t,r,b,rad_str,fill_rgba,stroke_t in drawable_to_layers(child_d):
                out['layers'].append((l+il,t+it,r+ir,b+ib,rad_str,fill_rgba,stroke_t))
            if child_d['gradient'] is not None and out['gradient'] is None:
                out['gradient']=child_d['gradient']
                if child_d['corner_radii'] is not None:out['corner_radii']=child_d['corner_radii']
                elif child_d['radius']!='0.0':out['radius']=child_d['radius']
            if child_d['ripple'] is not None and out['ripple'] is None:
                out['ripple']=child_d['ripple']
            if child_d['bg_image'] is not None and out['bg_image'] is None:
                out['bg_image']=child_d['bg_image']
            if child_d['animation_list'] is not None and out['animation_list'] is None:
                out['animation_list']=child_d['animation_list']
            if child_d['dynamic_drawable'] is not None and out['dynamic_drawable'] is None:
                out['dynamic_drawable']=child_d['dynamic_drawable']
            if child_d['animated_transitions'] and not out['animated_transitions']:
                out['animated_transitions']=list(child_d['animated_transitions'])
            if child_d['animated_vector'] is not None and out['animated_vector'] is None:
                out['animated_vector']=child_d['animated_vector']
            if child_d['auto_mirrored']:
                out['auto_mirrored']=True
            out['padding']=merge_padding(out['padding'],child_d['padding'])
            out['size']=[max(out['size'][0],child_d['size'][0]),max(out['size'][1],child_d['size'][1])]
        return out
    def parse_drawable_element(root, ref_label, stack=(), dname=None, src_path=None, allow_dynamic=False):
        if root.tag=='shape':
            return parse_shape_element(root,ref_label)
        if root.tag=='color':
            ca=strip_tools(root.attrib)
            if len(root) or set(ca)!={A+'color'}:raise ValueError('Unsupported color drawable: '+ref_label)
            rgba=static_color_rgba(ca[A+'color'])
            d=empty_drawable_dict();d['bg']=rgba_to_rust(rgba);d['bg_rgba']=rgba
            return d
        if root.tag=='inset':
            ia=strip_tools(root.attrib)
            base_in=float(margin_number(ia[A+'inset'])) if A+'inset' in ia else 0.0
            il=float(margin_number(ia[A+'insetLeft'])) if A+'insetLeft' in ia else float(margin_number(ia[A+'insetStart'])) if A+'insetStart' in ia else base_in
            ir=float(margin_number(ia[A+'insetRight'])) if A+'insetRight' in ia else float(margin_number(ia[A+'insetEnd'])) if A+'insetEnd' in ia else base_in
            it=float(margin_number(ia[A+'insetTop'])) if A+'insetTop' in ia else base_in
            ib=float(margin_number(ia[A+'insetBottom'])) if A+'insetBottom' in ia else base_in
            if A+'drawable' in ia and not len(root):
                inner=resolve_drawable_ref(ia[A+'drawable'],stack,allow_dynamic=allow_dynamic)
            elif A+'drawable' not in ia and len(root)==1:
                inner=parse_drawable_element(root[0],ref_label,stack,dname=dname,allow_dynamic=allow_dynamic)
            else:
                raise ValueError('Invalid inset drawable: '+ref_label)
            out=empty_drawable_dict()
            if inner['stateful_layers']:
                for req,forbid,(l,t,r,b,rad_str,fill_rgba,stroke_t) in inner['stateful_layers']:
                    out['stateful_layers'].append((req,forbid,(l+il,t+it,r+ir,b+ib,rad_str,fill_rgba,stroke_t)))
            else:
                for l,t,r,b,rad_str,fill_rgba,stroke_t in drawable_to_layers(inner):
                    out['layers'].append((l+il,t+it,r+ir,b+ib,rad_str,fill_rgba,stroke_t))
            out['padding']=inner['padding']
            out['size']=inner['size']
            out['ripple']=inner['ripple']
            out['bg_image']=inner['bg_image']
            out['stateful_bg_images']=list(inner['stateful_bg_images'])
            out['auto_mirrored']=inner['auto_mirrored']
            out['animation_list']=inner['animation_list']
            out['dynamic_drawable']=inner['dynamic_drawable']
            out['animated_transitions']=list(inner['animated_transitions'])
            out['animated_vector']=inner['animated_vector']
            if inner['gradient'] is not None:
                out['gradient']=inner['gradient']
                out['corner_radii']=inner['corner_radii']
                out['radius']=inner['radius']
            return out
        if root.tag=='layer-list':
            return flatten_layer_items(list(root),ref_label,stack,dname=dname,allow_dynamic=allow_dynamic)
        if root.tag=='ripple':
            ra=strip_tools(root.attrib)
            if A+'color' not in ra:raise ValueError('Ripple drawable requires android:color: '+ref_label)
            ripple_rgba=static_color_rgba(ra[A+'color'])
            has_mask=False;content_items=[]
            for child in root:
                if child.tag!='item':raise ValueError('Unsupported ripple child: '+ref_label)
                if child.attrib.get(A+'id')=='@android:id/mask':has_mask=True
                else:content_items.append(child)
            unbounded=(not has_mask) and (not content_items)
            ripple_anchor=rgba_to_csv(ripple_rgba)+('|unbounded' if unbounded else '')
            if not content_items:
                out=empty_drawable_dict();out['ripple']=ripple_anchor
                return out
            if len(content_items)==1:
                ci=content_items[0];cia=strip_tools(ci.attrib)
                has_inset=any(A+edge in cia and float(margin_number(cia[A+edge]))!=0.0 for edge in ('left','top','right','bottom','start','end'))
                if not has_inset:
                    if A+'drawable' in cia and not len(ci):
                        out=resolve_drawable_ref(cia[A+'drawable'],stack,allow_dynamic=allow_dynamic)
                    elif A+'drawable' not in cia and len(ci)==1:
                        out=parse_drawable_element(ci[0],ref_label,stack,dname=dname,allow_dynamic=allow_dynamic)
                    else:
                        raise ValueError('Invalid ripple content item: '+ref_label)
                    out['ripple']=ripple_anchor
                    return out
            out=flatten_layer_items(content_items,ref_label,stack,dname=dname,allow_dynamic=allow_dynamic)
            out['ripple']=ripple_anchor
            return out
        if root.tag in {'selector','animated-selector'}:
            sel_flags={'state_enabled':1,'state_pressed':2,'state_focused':4,'state_selected':8,'state_activated':16,'state_hovered':32,'state_checked':64}
            parsed_items=[]
            item_id_to_mask={}
            for idx,item in enumerate(root):
                if root.tag=='animated-selector' and item.tag=='transition':continue
                if item.tag!='item':raise ValueError('Invalid selector item: '+ref_label)
                ia=strip_tools(item.attrib)
                req=forbid=0;unknown_state=False
                for attr,raw_val in ia.items():
                    if attr in {A+'drawable',A+'color',A+'alpha',A+'id'}:continue
                    sname=attr[len(A):] if attr.startswith(A) else ''
                    if sname in sel_flags:
                        bv=resolve(raw_val)
                        if bv not in {'true','false'}:raise ValueError('Invalid selector boolean '+bv)
                        if bv=='true':req|=sel_flags[sname]
                        else:forbid|=sel_flags[sname]
                    else:
                        unknown_state=True
                if root.tag=='animated-selector' and A+'id' in ia:
                    raw_iid=resolve(ia[A+'id']).strip()
                    iid=id_key(raw_iid)
                    if iid is None:raise ValueError('Invalid animated-selector item id '+raw_iid)
                    if not unknown_state:item_id_to_mask[iid]=req
                if A+'drawable' in ia and not len(item):
                    item_d=resolve_drawable_ref(ia[A+'drawable'],stack)
                elif A+'color' in ia and not len(item):
                    r,g,b,al=static_color_rgba(ia[A+'color'])
                    if A+'alpha' in ia:al=int(al*float(resolve(ia[A+'alpha']))+0.5)
                    item_d=empty_drawable_dict();item_d['bg']=rgba_to_rust((r,g,b,al));item_d['bg_rgba']=(r,g,b,al)
                elif len(item)==1:
                    inline_key=f'{dname}__s{idx}' if dname else None
                    item_d=parse_drawable_element(item[0],ref_label,stack,dname=inline_key)
                else:
                    raise ValueError('Invalid selector item: '+ref_label)
                parsed_items.append((req,forbid,unknown_state,item_d))
            anim_transitions=[];tr_am=False
            if root.tag=='animated-selector' and allow_dynamic:
                anim_transitions,tr_am=decode_animated_selector_transitions(root,ref_label,dname or 'anim_sel',item_id_to_mask,stack)
            if not parsed_items:
                out=empty_drawable_dict()
                out['animated_transitions']=anim_transitions
                if tr_am:out['auto_mirrored']=True
                return out
            can_color_list=all(
                not unk and it_d['stroke_rgba'] is None and it_d['corner_radii'] is None
                and it_d['gradient'] is None and not it_d['layers'] and not it_d['stateful_layers']
                and it_d['ripple'] is None and it_d['bg_selector'] is None
                and it_d['bg_image'] is None and not it_d['stateful_bg_images']
                and it_d['animation_list'] is None
                for _,_,unk,it_d in parsed_items
            )
            opaque_radii={it_d['radius'] for _,_,_,it_d in parsed_items if it_d['bg_rgba'] is not None and it_d['bg_rgba'][3]>0}
            if can_color_list and len(opaque_radii)<=1:
                rules=[];has_default=False;pad=None;sz=[0.0,0.0]
                for req,forbid,_,it_d in parsed_items:
                    if req==0 and forbid==0:has_default=True
                    rgba=it_d['bg_rgba'] if it_d['bg_rgba'] is not None else (0,0,0,0)
                    rules.append(f'{req},{forbid},{rgba_to_csv(rgba)}')
                    pad=merge_padding(pad,it_d['padding'])
                    sz=[max(sz[0],it_d['size'][0]),max(sz[1],it_d['size'][1])]
                if not has_default:rules.append('0,0,0,0,0,0')
                out=empty_drawable_dict()
                out['bg_selector']=rules
                out['radius']=next(iter(opaque_radii)) if opaque_radii else '0.0'
                out['padding']=pad;out['size']=sz
                out['animated_transitions']=anim_transitions
                if tr_am:out['auto_mirrored']=True
                return out
            default_d=next((it_d for req,forbid,unk,it_d in parsed_items if req==0 and forbid==0 and not unk),None)
            if default_d is None:
                default_d=next((it_d for req,forbid,unk,it_d in parsed_items if req==1 and forbid==0 and not unk),None)
            out=dict(default_d) if default_d is not None else empty_drawable_dict()
            out['layers']=list(out['layers'])
            out['stateful_layers']=list(out['stateful_layers'])
            out['stateful_bg_images']=list(out['stateful_bg_images'])
            out['animated_transitions']=list(anim_transitions)
            pad=out['padding'];sz=list(out['size']);auto_m=bool(out['auto_mirrored']) or tr_am
            stateful=[]
            stateful_imgs=[]
            for req,forbid,unk,it_d in parsed_items:
                if unk:continue
                pad=merge_padding(pad,it_d['padding'])
                sz=[max(sz[0],it_d['size'][0]),max(sz[1],it_d['size'][1])]
                auto_m=auto_m or bool(it_d['auto_mirrored'])
                if it_d['bg_image'] is not None:
                    img_k,img_st=it_d['bg_image']
                    stateful_imgs.append((req,forbid,img_k,img_st))
                elif it_d['stateful_bg_images']:
                    for r2,f2,k2,st2 in it_d['stateful_bg_images']:
                        if ((req|r2)&(forbid|f2))==0:
                            stateful_imgs.append((req|r2,forbid|f2,k2,st2))
                elif it_d['bg_rgba'] is None and it_d['stroke_rgba'] is None and not it_d['layers'] and it_d['gradient'] is None and not it_d['stateful_layers'] and not it_d['bg_selector']:
                    stateful_imgs.append((req,forbid,'',''))
                item_layers=drawable_to_layers(it_d)
                if item_layers:
                    for layer in item_layers:
                        stateful.append((req,forbid,layer))
                else:
                    stateful.append((req,forbid,(0.0,0.0,0.0,0.0,'0.0',None,None)))
            if any(k for _,_,k,_ in stateful_imgs):
                out['stateful_bg_images']=stateful_imgs
                out['bg_image']=None
                out['padding']=pad
                out['size']=sz
                out['auto_mirrored']=auto_m
            has_visible_layer=any(layer[5] is not None or layer[6] is not None for _,_,layer in stateful)
            if stateful and has_visible_layer and any(req!=0 or forbid!=0 for req,forbid,_ in stateful):
                out['stateful_layers']=stateful
                out['layers']=[]
                out['padding']=pad
                out['size']=sz
            if auto_m:
                out['auto_mirrored']=True
            return out
        if root.tag in {'rotate','scale','clip'}:
            kind,payload,auto_m=validate_drawable_wrapper(root,ref_label)
            out=resolve_drawable_ref(payload,stack) if kind=='ref' else parse_drawable_element(payload,ref_label,stack,dname=dname)
            if auto_m:out['auto_mirrored']=True
            if allow_dynamic:out['dynamic_drawable']=decode_dynamic_drawable(root,ref_label,dname or 'dyn',stack)
            return out
        if root.tag=='animation-list':
            chosen=validate_animation_list(root,ref_label)
            cia=strip_tools(chosen.attrib)
            if A+'drawable' in cia and not len(chosen):
                out=resolve_drawable_ref(cia[A+'drawable'],stack)
            elif A+'drawable' not in cia and len(chosen)==1:
                out=parse_drawable_element(chosen[0],ref_label,stack,dname=dname)
            else:
                raise ValueError('Invalid animation-list item: '+ref_label)
            anim_spec,_,_,any_am=decode_animation_list_frames(root,ref_label,dname or 'anim_list',stack)
            out['animation_list']=anim_spec
            if any_am:out['auto_mirrored']=True
            return out
        if root.tag in {'level-list','transition'}:
            kind,payload,auto_m=select_list_drawable_item(root,ref_label)
            out=resolve_drawable_ref(payload,stack) if kind=='ref' else parse_drawable_element(payload,ref_label,stack,dname=dname)
            if auto_m:out['auto_mirrored']=True
            if allow_dynamic:out['dynamic_drawable']=decode_dynamic_drawable(root,ref_label,dname or 'dyn',stack)
            return out
        if root.tag=='vector':
            if dname is not None:
                vw,vh,_,_,auto_m=resolve_and_rasterize_vector(src_path if src_path is not None else root,dname)
                out=empty_drawable_dict();out['size']=[vw,vh];out['bg_image']=(dname,'');out['auto_mirrored']=auto_m
                return out
            return empty_drawable_dict()
        if root.tag=='animated-vector':
            avd_spec,first_key,vw,vh,_,auto_m=decode_animated_vector_frames(root,ref_label,dname or 'avd',stack,sample_frames=allow_dynamic)
            out=empty_drawable_dict();out['size']=[vw,vh];out['bg_image']=(first_key,'');out['auto_mirrored']=auto_m
            if allow_dynamic and avd_spec is not None:out['animated_vector']=avd_spec
            return out
        if root.tag=='bitmap':
            ba=strip_tools(root.attrib)
            allowed_bmp={A+'src',A+'tint',A+'tintMode',A+'gravity',A+'tileMode',A+'tileModeX',A+'tileModeY',A+'antialias',A+'dither',A+'filter',A+'mipMap',A+'alpha',A+'autoMirrored',A+'visible'}
            if len(root) or (set(ba)-allowed_bmp) or A+'src' not in ba:
                raise ValueError('Invalid bitmap drawable: '+ref_label)
            if A+'gravity' in ba:
                gparts=[p.strip() for p in resolve(ba[A+'gravity']).split('|') if p.strip()]
                if not gparts or any(p not in BITMAP_GRAVITY_TOKENS for p in gparts):
                    raise ValueError('Unsupported bitmap gravity '+ba[A+'gravity'])
            if A+'tintMode' in ba:
                tm=resolve(ba[A+'tintMode']).strip()
                if tm not in {'src_atop','src_in','src_over','multiply','screen','add'}:
                    raise ValueError('Unsupported bitmap tintMode '+tm)
            bmp_tint_rgba=static_color_rgba(ba[A+'tint']) if A+'tint' in ba else None
            src_ref=resolve(ba[A+'src'])
            if not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',src_ref):
                raise ValueError('Unsupported bitmap src '+src_ref)
            if src_ref in stack:raise ValueError('Cyclic drawable reference: '+' -> '.join(stack+(src_ref,)))
            if not resource_roots:raise ValueError('Drawable needs --resources: '+src_ref)
            src_name=src_ref.split('/')[-1]
            cands,nodpi,xfile=find_drawable_candidates(src_name)
            out=empty_drawable_dict()
            img_key=dname or src_name
            if xfile is not None and not cands and nodpi is None:
                out=parse_drawable_element(ET.parse(xfile).getroot(),src_ref,stack+(src_ref,),dname=img_key,src_path=xfile)
            elif cands or nodpi is not None:
                if len({dpi for dpi,_ in cands})!=len(cands):raise ValueError('Ambiguous PNG drawable density '+src_ref)
                if cands:
                    higher=sorted((dpi,f) for dpi,f in cands if dpi>=density)
                    sdpi,img_file=higher[0] if higher else max(cands)
                else:
                    sdpi,img_file=160,nodpi
                w,h,pixels,scale,pad_dp,stretch_spec=decode_drawable_image(img_file,sdpi)
                if img_key not in images:images[img_key]=(w,h,pixels,scale)
                out['padding']=pad_dp
                out['size']=[round(w*scale,4),round(h*scale,4)]
                out['bg_image']=(img_key,stretch_spec)
            else:
                raise ValueError('Unresolved drawable '+src_ref)
            if bmp_tint_rgba is not None:
                r,g,b,al=bmp_tint_rgba
                indicator_tints[img_key]=f'#{al:02x}{r:02x}{g:02x}{b:02x}'
                out['bitmap_tint']=bmp_tint_rgba
            return out
        raise ValueError('Only solid shape drawables supported: '+ref_label)
    def background(v):
        return resolve_drawable_ref(v,allow_dynamic=True)
    def divider_drawable(ref, horizontal):
        if ref=='@null':return 'None','0.0'
        if not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',ref) or not resource_roots:raise ValueError('Divider needs local shape drawable: '+ref)
        path=find_drawable_xml(ref.split('/')[-1])
        if path is None:raise ValueError('Unresolved divider '+ref)
        root=ET.parse(path).getroot()
        if root.tag=='layer-list' and len(root)==1 and root[0].tag=='item' and len(root[0])==1:
            root=root[0][0]
        if root.tag=='shape' and root.attrib.get(A+'shape','rectangle')=='line':
            sd=parse_shape_element(root,ref)
            if sd['stroke_rgba'] is None:raise ValueError('Line divider requires stroke: '+ref)
            return sd['stroke'],sd['stroke_width']
        if root.tag!='shape' or root.attrib.get(A+'shape','rectangle')!='rectangle' or any(c.tag not in {'solid','size','padding'} for c in root):raise ValueError('Only solid rectangular divider drawables supported: '+ref)
        solid=root.find('solid');size_node=root.find('size');axis='width' if horizontal else 'height'
        if solid is None or size_node is None or A+axis not in size_node.attrib:raise ValueError('Divider requires solid color and '+axis+': '+ref)
        return color(solid.attrib[A+'color']),number(size_node.attrib[A+axis])
    def parse_interpolator_kind(raw_curve, err_prefix):
        curve=resolve(raw_curve).strip()
        if curve in {'@android:anim/linear_interpolator','@android:interpolator/linear'}:
            return 'linear'
        if curve in {
            '@android:anim/accelerate_interpolator',
            '@android:interpolator/accelerate_cubic',
            '@android:interpolator/accelerate_quad',
            '@android:interpolator/accelerate_quint',
            '@android:interpolator/fast_out_linear_in',
        }:
            return 'accelerate'
        if curve in {
            '@android:anim/decelerate_interpolator',
            '@android:interpolator/decelerate_cubic',
            '@android:interpolator/decelerate_quad',
            '@android:interpolator/decelerate_quint',
            '@android:interpolator/linear_out_slow_in',
        }:
            return 'decelerate'
        if curve in {
            '@android:interpolator/fast_out_slow_in',
            '@android:interpolator/fast_out_extra_slow_in',
        }:
            return 'fast_out_slow_in'
        if curve in {
            '@android:anim/overshoot_interpolator',
            '@android:interpolator/overshoot',
        }:
            return 'overshoot'
        if curve in {
            '@android:anim/anticipate_interpolator',
            '@android:interpolator/anticipate',
        }:
            return 'anticipate'
        if curve in {
            '@android:anim/accelerate_decelerate_interpolator',
            '@android:anim/anticipate_overshoot_interpolator',
            '@android:anim/bounce_interpolator',
            '@android:anim/cycle_interpolator',
            '@android:interpolator/accelerate_decelerate',
            '@android:interpolator/anticipate_overshoot',
            '@android:interpolator/bounce',
        }:
            return 'accelerate_decelerate'
        if re.fullmatch(r'@interpolator/[a-z][a-z0-9_]*',curve):
            if resource_roots:
                ifile=find_resource_file('interpolator',curve.split('/')[-1]+'.xml')
                if ifile is not None:
                    iroot=ET.parse(ifile).getroot()
                    if iroot.tag not in {'linearInterpolator','accelerateInterpolator','decelerateInterpolator','accelerateDecelerateInterpolator','overshootInterpolator','anticipateInterpolator','anticipateOvershootInterpolator','bounceInterpolator','cycleInterpolator','pathInterpolator'}:
                        raise ValueError(err_prefix+curve)
                    if iroot.tag=='linearInterpolator':return 'linear'
                    if iroot.tag=='accelerateInterpolator':return 'accelerate'
                    if iroot.tag=='decelerateInterpolator':return 'decelerate'
                    if iroot.tag=='overshootInterpolator':return 'overshoot'
                    if iroot.tag=='anticipateInterpolator':return 'anticipate'
                    return 'accelerate_decelerate'
            return 'fast_out_slow_in' if 'emphasized' in curve else 'accelerate_decelerate'
        raise ValueError(err_prefix+curve)
    def parse_interpolator_linear(raw_curve, err_prefix):
        return parse_interpolator_kind(raw_curve,err_prefix)=='linear'
    def fmt_anim_num(v):
        if v==0.0:return '0'
        if v.is_integer():return str(int(v))
        return str(round(v,6)).rstrip('0').rstrip('.')
    def object_alpha_animator(ref):
        if not re.fullmatch(r'@animator/[a-z][a-z0-9_]*',ref) or not resource_roots:raise ValueError('Animator needs local @animator resource: '+ref)
        path=find_resource_file('animator',ref.split('/')[-1]+'.xml')
        if path is None:raise ValueError('Unresolved animator '+ref)
        def parse(root, base_offset_ms=0):
            if root.tag=='set':
                sa=strip_tools(root.attrib)
                if set(sa)-{A+'ordering'}:raise ValueError('Unsupported AnimatorSet attribute: '+ref)
                ordering=resolve(sa.get(A+'ordering','together')).strip()
                if ordering not in {'together','sequentially'}:raise ValueError('Unsupported AnimatorSet ordering: '+ordering)
                properties={}
                if ordering=='together':
                    max_dur=0
                    for child in root:
                        child_properties,child_dur=parse(child,base_offset_ms)
                        if properties.keys() & child_properties.keys():raise ValueError('Duplicate animator property: '+ref)
                        properties.update(child_properties)
                        if child_dur>max_dur:max_dur=child_dur
                    if not properties:raise ValueError('Empty AnimatorSet: '+ref)
                    return properties,max_dur
                cursor=0
                has_any=False
                for child in root:
                    child_properties,child_dur=parse(child,base_offset_ms+cursor)
                    has_any=True
                    cursor+=child_dur
                    for pk,pv in child_properties.items():
                        if pk not in properties:properties[pk]=pv
                if not has_any or not properties:raise ValueError('Empty AnimatorSet: '+ref)
                return properties,cursor
            ra=strip_tools(root.attrib)
            allowed={A+'propertyName',A+'valueFrom',A+'valueTo',A+'duration',A+'startOffset',A+'interpolator',A+'valueType',A+'repeatCount',A+'repeatMode'}
            prop=ra.get(A+'propertyName')
            supported_props={'alpha','translationX','translationY','scaleX','scaleY','rotation','elevation','translationZ'}
            if root.tag!='objectAnimator' or len(root) or set(ra)-allowed or prop not in supported_props:raise ValueError('Unsupported objectAnimator resource: '+ref)
            if resolve(ra.get(A+'valueType','floatType')).strip()!='floatType':raise ValueError('Unsupported animator valueType')
            try:
                duration=int(resolve(ra.get(A+'duration','300')).strip())
                offset=int(resolve(ra.get(A+'startOffset','0')).strip())
            except ValueError:raise ValueError('Invalid animator duration '+ref)
            if not 0<=duration<=60000 or not 0<=offset<=60000 or base_offset_ms+offset>60000:raise ValueError('Invalid animator duration or startOffset '+ref)
            if A+'repeatCount' in ra:
                rc=resolve(ra[A+'repeatCount']).strip()
                if rc!='infinite' and not re.fullmatch(r'-1|\d+',rc):raise ValueError('Invalid animator repeatCount '+rc)
            if A+'repeatMode' in ra:
                rm=resolve(ra[A+'repeatMode']).strip()
                if rm not in {'restart','reverse'}:raise ValueError('Invalid animator repeatMode '+rm)
            curve=ra.get(A+'interpolator','@android:anim/accelerate_decelerate_interpolator')
            interp=parse_interpolator_kind(curve,'Unsupported animator interpolator ')
            linear=(interp=='linear')
            def value(name, optional_default=None):
                if A+name not in ra:
                    if optional_default is not None:return optional_default
                    raise ValueError('Invalid animator value '+ref)
                raw=resolve(ra[A+name]).strip()
                match=re.fullmatch(r'(-?(?:\d+(?:\.\d*)?|\.\d+))(dp|dip|px)?',raw)
                if not match:raise ValueError('Invalid animator value '+ref)
                v=float(match.group(1));unit=match.group(2)
                if prop in {'alpha','scaleX','scaleY','rotation'}:
                    if unit is not None and prop!='alpha':raise ValueError('Invalid animator value '+ref)
                    return v
                return v if unit in {'dp','dip'} else v*160.0/density
            default_from=None if prop in {'alpha','translationX','translationY'} else (1.0 if prop in {'scaleX','scaleY'} else 0.0)
            start=value('valueFrom',default_from);end=value('valueTo')
            if prop=='alpha' and not (0<=start<=1 and 0<=end<=1):raise ValueError('Invalid alpha animator '+ref)
            return {prop:(start,end,duration/1000.0,linear,(base_offset_ms+offset)/1000.0,duration,base_offset_ms+offset,interp)},offset+duration
        properties,_=parse(ET.parse(path).getroot(),0)
        alpha=translate='None'
        scale=rotate=elev_anim=None
        if 'alpha' in properties:
            start,end,duration,linear,offset=properties['alpha'][:5]
            alpha='Some(AlphaAnimation { from:'+str(start)+', to:'+str(end)+', duration:'+str(duration)+', start_offset:'+str(offset)+', linear:'+str(linear).lower()+' })'
        x=properties.get('translationX');y=properties.get('translationY')
        if x or y:
            duration,linear,offset=(x or y)[2:5]
            values=list(x[:2] if x else (0.0,0.0))+list(y[:2] if y else (0.0,0.0))
            deltas=','.join('AnimationDelta { value:'+str(v)+', mode:0 }' for v in values)
            y_timing='Some(('+str(y[2])+','+str(y[3]).lower()+','+str(y[4])+'))' if y and x and y[2:5]!=x[2:5] else 'None'
            translate='Some(TranslateAnimation { deltas:['+deltas+'], duration:'+str(duration)+', start_offset:'+str(offset)+', linear:'+str(linear).lower()+', y_timing:'+y_timing+' })'
        sx=properties.get('scaleX');sy=properties.get('scaleY')
        if sx or sy:
            s_dur,s_lin,s_off=(sx or sy)[2:5]
            fsx,tsx=sx[:2] if sx else (1.0,1.0)
            fsy,tsy=sy[:2] if sy else (1.0,1.0)
            scale=f'{0.0 if fsx==0.0 else fsx},{0.0 if tsx==0.0 else tsx},{0.0 if fsy==0.0 else fsy},{0.0 if tsy==0.0 else tsy}|center,center|{s_dur}|{s_off}|{"1" if s_lin else "0"}'
        rot=properties.get('rotation')
        if rot:
            rf,rt,r_dur,r_lin,r_off=rot[:5]
            rotate=f'{fmt_anim_num(rf)},{fmt_anim_num(rt)}|center,center|{r_dur}|{r_off}|{"1" if r_lin else "0"}'
        el=properties.get('elevation');tz=properties.get('translationZ')
        if el and tz:
            ef,et,_,_,_,e_dur,e_off,e_interp=el
            zf,zt,_,_,_,z_dur,z_off,z_interp=tz
            if (e_dur,e_off,e_interp)==(z_dur,z_off,z_interp):
                elev_anim=f'{fmt_anim_num(ef+zf)},{fmt_anim_num(et+zt)}|{e_dur}|{e_off}|{e_interp}'
            else:
                elev_anim=f'{fmt_anim_num(ef)},{fmt_anim_num(et)}|{e_dur}|{e_off}|{e_interp}'
        elif el or tz:
            ef,et,_,_,_,e_dur,e_off,e_interp=(el or tz)
            elev_anim=f'{fmt_anim_num(ef)},{fmt_anim_num(et)}|{e_dur}|{e_off}|{e_interp}'
        return alpha,translate,scale,rotate,elev_anim
    def alpha_animation(ref):
        if not re.fullmatch(r'@anim/[a-z][a-z0-9_]*',ref) or not resource_roots:raise ValueError('Animation needs local @anim resource: '+ref)
        path=find_resource_file('anim',ref.split('/')[-1]+'.xml')
        if path is None:raise ValueError('Unresolved animation '+ref)
        root=ET.parse(path).getroot()
        def parse_pivot(raw):
            v=resolve(raw).strip()
            m=re.fullmatch(r'(-?(?:\d+(?:\.\d+)?|\.\d+))(%p|%P|%|dp|dip|px)?',v)
            if not m:raise ValueError('Invalid animation pivot '+raw)
            num=float(m.group(1));suf=m.group(2)
            if suf in {'%p','%P'}:return f'{fmt_anim_num(num)}%p'
            if suf=='%':return f'{fmt_anim_num(num)}%'
            if suf in {'dp','dip'}:return f'{fmt_anim_num(num)}dp'
            dp_v=round(num*160.0/density,4)
            return f'{fmt_anim_num(dp_v)}dp'
        def validate_common_extras(aa):
            for bk in (A+'fillBefore',A+'fillAfter',A+'fillEnabled',A+'detachWallpaper'):
                if bk in aa and resolve(aa[bk]).strip() not in {'true','false'}:raise ValueError('Invalid animation boolean '+aa[bk])
            if A+'repeatCount' in aa:
                rc=resolve(aa[A+'repeatCount']).strip()
                if rc!='infinite' and not re.fullmatch(r'-1|\d+',rc):raise ValueError('Invalid animation repeatCount '+rc)
            if A+'repeatMode' in aa:
                rm=resolve(aa[A+'repeatMode']).strip()
                if rm not in {'restart','reverse'}:raise ValueError('Invalid animation repeatMode '+rm)
            if A+'zAdjustment' in aa:
                za=resolve(aa[A+'zAdjustment']).strip()
                if za not in {'normal','top','bottom'}:raise ValueError('Invalid animation zAdjustment '+za)
        def parse(anim, inherited_curve=None, inherited_duration=None, parent_offset=0):
            aa=strip_tools(anim.attrib)
            common={A+'duration',A+'interpolator',A+'startOffset',A+'fillBefore',A+'fillAfter',A+'fillEnabled',A+'repeatCount',A+'repeatMode',A+'zAdjustment',A+'detachWallpaper'}
            try: offset=int(resolve(aa.get(A+'startOffset','0')).strip())
            except ValueError:raise ValueError('Invalid animation startOffset '+ref)
            if not 0<=offset<=60000 or parent_offset+offset>60000:raise ValueError('Invalid animation startOffset '+ref)
            offset+=parent_offset
            if anim.tag=='set':
                if set(aa)-(common | {A+'shareInterpolator'}):raise ValueError('Unsupported animation set attributes: '+ref)
                validate_common_extras(aa)
                share=resolve(aa.get(A+'shareInterpolator','true')).strip()
                if share not in {'true','false'}:raise ValueError('Invalid shareInterpolator '+share)
                if A+'interpolator' in aa:parse_interpolator_linear(aa[A+'interpolator'],'Unsupported animation interpolator ')
                curve=inherited_curve or (aa.get(A+'interpolator','@android:anim/accelerate_decelerate_interpolator') if share=='true' else None)
                duration=inherited_duration if inherited_duration is not None else aa.get(A+'duration')
                alpha=translate='None'
                scale=rotate=None
                for child in anim:
                    a,t,s,r=parse(child,curve,duration,offset)
                    if a!='None' and alpha!='None' or t!='None' and translate!='None' or s is not None and scale is not None or r is not None and rotate is not None:raise ValueError('Duplicate animation type in set: '+ref)
                    if a!='None':alpha=a
                    if t!='None':translate=t
                    if s is not None:scale=s
                    if r is not None:rotate=r
                return alpha,translate,scale,rotate
            tag_extras={
                'alpha':{A+'fromAlpha',A+'toAlpha'},
                'translate':{A+'fromXDelta',A+'toXDelta',A+'fromYDelta',A+'toYDelta'},
                'scale':{A+'fromXScale',A+'toXScale',A+'fromYScale',A+'toYScale',A+'pivotX',A+'pivotY'},
                'rotate':{A+'fromDegrees',A+'toDegrees',A+'pivotX',A+'pivotY'},
            }
            if anim.tag not in tag_extras or len(anim) or set(aa)-(common | tag_extras[anim.tag]):raise ValueError('Unsupported animation resource: '+ref)
            validate_common_extras(aa)
            try:
                duration=int(resolve(inherited_duration if inherited_duration is not None else aa.get(A+'duration','0')).strip())
            except ValueError:raise ValueError('Invalid animation '+ref)
            if not 0<=duration<=60000:raise ValueError('Invalid animation '+ref)
            curve=inherited_curve or aa.get(A+'interpolator','@android:anim/accelerate_decelerate_interpolator')
            is_linear=parse_interpolator_linear(curve,'Unsupported animation interpolator ')
            linear=str(is_linear).lower()
            lin_flag='1' if is_linear else '0'
            if anim.tag=='alpha':
                try:
                    start=float(resolve(aa.get(A+'fromAlpha','1')).strip()); end=float(resolve(aa.get(A+'toAlpha','1')).strip())
                except ValueError:raise ValueError('Invalid animation '+ref)
                if not (0<=start<=1 and 0<=end<=1):raise ValueError('Invalid animation '+ref)
                return ('Some(AlphaAnimation { from:'+str(start)+', to:'+str(end)+', duration:'+str(duration/1000.0)+', start_offset:'+str(offset/1000.0)+', linear:'+linear+' })','None',None,None)
            if anim.tag=='scale':
                def parse_scale_val(sk):
                    if sk not in aa:return '1.0'
                    sv=resolve(aa[sk]).strip()
                    if not re.fullmatch(r'-?(?:\d+(?:\.\d+)?|\.\d+)%?',sv):raise ValueError('Invalid scale animation value '+sv)
                    fv=round(float(sv[:-1])/100.0,6) if sv.endswith('%') else float(sv)
                    return str(0.0 if fv==0.0 else fv)
                fsx=parse_scale_val(A+'fromXScale')
                tsx=parse_scale_val(A+'toXScale')
                fsy=parse_scale_val(A+'fromYScale')
                tsy=parse_scale_val(A+'toYScale')
                px=parse_pivot(aa[A+'pivotX']) if A+'pivotX' in aa else '0dp'
                py=parse_pivot(aa[A+'pivotY']) if A+'pivotY' in aa else '0dp'
                return ('None','None',f'{fsx},{tsx},{fsy},{tsy}|{px},{py}|{duration/1000.0}|{offset/1000.0}|{lin_flag}',None)
            if anim.tag=='rotate':
                def parse_deg(rk):
                    if rk not in aa:return '0'
                    rv=resolve(aa[rk]).strip()
                    if not re.fullmatch(r'-?(?:\d+(?:\.\d+)?|\.\d+)',rv):raise ValueError('Invalid rotate animation degrees '+rv)
                    return fmt_anim_num(float(rv))
                fd=parse_deg(A+'fromDegrees')
                td=parse_deg(A+'toDegrees')
                px=parse_pivot(aa[A+'pivotX']) if A+'pivotX' in aa else '0dp'
                py=parse_pivot(aa[A+'pivotY']) if A+'pivotY' in aa else '0dp'
                return ('None','None',None,f'{fd},{td}|{px},{py}|{duration/1000.0}|{offset/1000.0}|{lin_flag}')
            def delta(name):
                raw=resolve(aa.get(A+name,'0')).strip()
                match=re.fullmatch(r'(-?\d+(?:\.\d+)?)(%p|%|dp|px)?',raw)
                if not match:raise ValueError('Unsupported animation delta '+raw)
                value=float(match.group(1));suffix=match.group(2)
                return 'AnimationDelta { value:'+str(value/100 if suffix in {'%','%p'} else value)+', mode:'+str(2 if suffix=='%p' else 1 if suffix=='%' else 0)+' }'
            values=','.join(delta(x) for x in ('fromXDelta','toXDelta','fromYDelta','toYDelta'))
            return ('None','Some(TranslateAnimation { deltas:['+values+'], duration:'+str(duration/1000.0)+', start_offset:'+str(offset/1000.0)+', linear:'+linear+', y_timing:None })',None,None)
        return parse(root)+(None,)
    def id_key(raw):
        match=re.fullmatch(r'@(?:(android):|\+)?id/([a-zA-Z_][a-zA-Z0-9_]*)',raw)
        if not match:return None
        if match.group(1):
            if match.group(2) not in ANDROID_IDS:raise ValueError('Unsupported Android ID '+raw)
            return 'android:'+match.group(2)
        return match.group(2)
    def parse_referenced_ids(raw):
        val=resolve(raw).strip()
        if not val:return []
        out=[]
        for part in val.split(','):
            tok=part.strip()
            if not tok:continue
            if tok.startswith('@'):
                key=id_key(tok)
                if key is None:raise ValueError('Invalid constraint_referenced_ids entry '+tok)
                out.append(key)
            else:
                out.append(tok)
        return out
    def validate_relative_cycles(root):
        for container in (n for n in root.iter() if n.tag in {'RelativeLayout','TwoLineListItem'}):
            children=list(container)
            by_id={(id_key(c.attrib[A+'id']) if c.attrib[A+'id'].startswith('@') else c.attrib[A+'id']):i for i,c in enumerate(children) if c.attrib.get(A+'id')}
            for axis in ({0,1,4,5,8,9,10,11},{2,3,6,7,22}):
                for rtl in (False,True):
                    graph=[set() for _ in children]
                    for i,child in enumerate(children):
                        values={kind:child.attrib[A+name] for name,kind in RELATIVE_ANCHORS.items() if A+name in child.attrib and kind in axis}
                        if 8 in values or 9 in values:values.pop(0,None);values.pop(1,None)
                        if 10 in values or 11 in values:values.pop(4,None);values.pop(5,None)
                        for raw in values.values():
                            key=id_key(resolve(raw))
                            j=by_id.get(key) if key else None
                            if j is not None and j!=i:graph[i].add(j)
                    status=[0]*len(children)
                    def visit(i):
                        if status[i]==1:raise ValueError('Circular RelativeLayout dependencies')
                        if status[i]==2:return
                        status[i]=1
                        for j in graph[i]:visit(j)
                        status[i]=2
                    for i in range(len(children)):visit(i)
    ids=set()
    def resolve_animation_list_drawable(ref, seen=()):
        if not ref or ref=='@null' or not resource_roots:return None
        if not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',ref):return None
        if ref in seen:raise ValueError('Cyclic drawable reference '+ref)
        dname=ref.split('/')[-1]
        xml_path=find_drawable_xml(dname)
        if xml_path is None:return None
        root=ET.parse(xml_path).getroot()
        def unwrap_el(el, cur_ref, cur_key, cur_seen):
            if el.tag=='animation-list':
                return (el, cur_ref, cur_key, cur_seen)
            if el.tag=='inset':
                ia=strip_tools(el.attrib)
                for ik in ('inset','insetLeft','insetRight','insetTop','insetBottom','insetStart','insetEnd'):
                    if A+ik in ia:margin_number(ia[A+ik])
                if A+'drawable' in ia and not len(el):
                    return resolve_animation_list_drawable(resolve(ia[A+'drawable']), cur_seen+(cur_ref,))
                if A+'drawable' not in ia and len(el)==1:
                    return unwrap_el(el[0], cur_ref, cur_key, cur_seen)
                raise ValueError('Invalid inset drawable: '+cur_ref)
            return None
        return unwrap_el(root, ref, dname, seen)
    def indicator_resources(ref,group,seen=()):
        if ref=='@null':return ['0,0,']
        if ref in seen:raise ValueError('Cyclic drawable reference '+ref)
        if not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',ref) or not resource_roots:raise ValueError('Invalid indicator drawable '+ref)
        name=ref.split('/')[-1];selector=find_drawable_xml(name)
        def drawable(value):
            if value=='@null':return ''
            if not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',value):raise ValueError('Invalid indicator image '+value)
            image=value.split('/')[-1]
            candidates,_,vector=find_drawable_candidates(image)
            if len({dpi for dpi,_ in candidates})!=len(candidates):raise ValueError('Ambiguous indicator drawable '+value)
            if candidates and vector is not None:raise ValueError('Ambiguous indicator drawable '+value)
            if not candidates and vector is not None:
                resolve_and_rasterize_vector(vector,image)
                return image
            if not candidates:raise ValueError('Unresolved indicator drawable '+value)
            higher=sorted((dpi,file) for dpi,file in candidates if dpi>=density);dpi,file=higher[0] if higher else max(candidates)
            if image not in images:
                w,h,pixels,scale,_,_=decode_drawable_image(file,dpi)
                images[image]=(w,h,pixels,scale)
            return image
        if selector is None:return ['0,0,'+drawable(ref)]
        root=ET.parse(selector).getroot()
        if root.tag=='vector':return ['0,0,'+drawable(ref)]
        if root.tag=='inset':
            ia=strip_tools(root.attrib)
            if A+'drawable' in ia and not len(root):return indicator_resources(resolve(ia[A+'drawable']),group,seen+(ref,))
            raise ValueError('Invalid inset drawable: '+ref)
        if root.tag in {'rotate','scale','clip'}:
            kind,payload,_=validate_drawable_wrapper(root,ref)
            if kind=='ref':return ['0,0,'+drawable(payload)]
            raise ValueError('Unsupported indicator selector '+ref)
        if root.tag in {'level-list','transition'}:
            kind,payload,_=select_list_drawable_item(root,ref)
            if kind=='ref':return ['0,0,'+drawable(payload)]
            raise ValueError('Unsupported indicator selector '+ref)
        if root.tag=='animation-list':
            chosen=validate_animation_list(root,ref)
            cia=strip_tools(chosen.attrib)
            if A+'drawable' in cia and not len(chosen):return ['0,0,'+drawable(resolve(cia[A+'drawable']))]
            raise ValueError('Invalid animation-list item: '+ref)
        if root.tag not in {'selector','animated-selector'} or (root.tag=='selector' and root.attrib):raise ValueError('Unsupported indicator selector '+ref)
        states={'state_expanded':1,'state_empty':2} if group else {'state_last':1}
        entries=[]
        for item in root:
            if root.tag=='animated-selector' and item.tag=='transition':continue
            allowed={A+'drawable'}|{A+state for state in states}|({A+'id'} if root.tag=='animated-selector' else set())
            if item.tag!='item' or len(item) or set(strip_tools(item.attrib))-allowed or A+'drawable' not in item.attrib:raise ValueError('Unsupported indicator selector item')
            required=forbidden=0
            for state,bit in states.items():
                value=item.get(A+state)
                if value not in {None,'true','false'}:raise ValueError('Invalid indicator state')
                if value=='true':required|=bit
                if value=='false':forbidden|=bit
            entries.append(str(required)+','+str(forbidden)+','+drawable(item.get(A+'drawable')))
        if not entries:raise ValueError('Empty indicator selector')
        return entries
    def node(n,parent=None,parent_node=None):
        if n.tag not in TAGS:raise ValueError('Unsupported Android view: '+n.tag)
        material_divider=n.tag=='com.google.android.material.divider.MaterialDivider'
        material_card=n.tag=='com.google.android.material.card.MaterialCardView'
        for k in n.attrib:
            if material_card and k==APP+'bhooCardVariant' and n.attrib[k] in {'Outlined','Filled','Elevated'}:continue
            if material_card and k.startswith(APP) and k[len(APP):] in MATERIAL_CARD_ATTRS:continue
            if material_card and k==A+'checkable':continue
            if material_divider and k.startswith(APP) and k[len(APP):] in MATERIAL_DIVIDER_ATTRS:continue
            if n.tag=='Toolbar' and k.startswith(APP) and k[len(APP):] in (ATTRS_EXTRA_TOOLBAR | TOOLBAR_APP_EXTRA_ATTRS):continue
            if n.tag=='Button' and k.startswith(APP) and k[len(APP):] in APP_BUTTON_ATTRS:continue
            if k.startswith(APP) and k[len(APP):] in APP_COMMON_ATTRS:continue
            if k.startswith(APP) and k[len(APP):] in (CONSTRAINT_ANCHORS.keys() | CONSTRAINT_BIASES.keys()):continue
            if not k.startswith(A) or k[len(A):] not in ATTRS:raise ValueError('Unsupported attribute on '+n.tag+': '+k)
        null_preserving={'tabStripLeft','tabStripRight','groupIndicator','childIndicator','checkMark','button','thumb','tickMark','drawableLeft','drawableRight','drawableTop','drawableBottom','drawableStart','drawableEnd','background','foreground','src','navigationIcon','divider'}
        a={k[len(A):]:(v if k[len(A):] in null_preserving and v=='@null' else resolve(v)) for k,v in n.attrib.items() if k.startswith(A)}
        for app_key,android_key in (
            ('srcCompat','src'),('tint','tint'),('backgroundTint','backgroundTint'),
            ('drawableStartCompat','drawableStart'),('drawableEndCompat','drawableEnd'),
            ('drawableTopCompat','drawableTop'),('drawableBottomCompat','drawableBottom'),
            ('drawableLeftCompat','drawableLeft'),('drawableRightCompat','drawableRight'),
            ('drawableTint','drawableTint'),('drawableTintMode','drawableTintMode'),
            ('fontFamily','fontFamily'),('lineHeight','lineHeight'),('elevation','elevation'),('tooltipText','tooltipText'),
            ('autoSizeTextType','autoSizeTextType'),('autoSizeMinTextSize','autoSizeMinTextSize'),
            ('autoSizeMaxTextSize','autoSizeMaxTextSize'),('autoSizeStepGranularity','autoSizeStepGranularity'),
        ):
            if APP+app_key in n.attrib and android_key not in a:
                raw=n.attrib[APP+app_key]
                a[android_key]=raw if android_key in null_preserving and raw=='@null' else resolve(raw)
        if n.tag=='Button':
            if APP+'icon' in n.attrib and 'drawableStart' not in a:
                raw=n.attrib[APP+'icon']
                a['drawableStart']=raw if raw=='@null' else resolve(raw)
            if APP+'iconTint' in n.attrib and 'drawableTint' not in a:
                raw=resolve(n.attrib[APP+'iconTint'])
                if raw!='@null':a['drawableTint']=raw
            if APP+'iconPadding' in n.attrib and 'drawablePadding' not in a:
                a['drawablePadding']=resolve(n.attrib[APP+'iconPadding'])
            if APP+'iconGravity' in n.attrib:
                ig=resolve(n.attrib[APP+'iconGravity']).strip()
                if ig not in {'start','textStart','end','textEnd','top','textTop'}:raise ValueError('Unsupported iconGravity '+ig)
            if APP+'iconSize' in n.attrib:
                number(n.attrib[APP+'iconSize'])
        if n.tag=='Toolbar':
            for key in ATTRS_EXTRA_TOOLBAR:
                if APP+key in n.attrib and key not in a:
                    raw=n.attrib[APP+key]
                    a[key]=raw if key in null_preserving and raw=='@null' else resolve(raw)
            for ck in ('titleCentered','subtitleCentered'):
                if APP+ck in n.attrib:
                    cv=resolve(n.attrib[APP+ck]).strip()
                    if cv not in {'true','false'}:raise ValueError('Invalid '+ck+' '+cv)
            if APP+'navigationIconTint' in n.attrib:
                nt=resolve(n.attrib[APP+'navigationIconTint']).strip()
                if nt!='@null':static_color_rgba(nt)
        relative=[]
        compound_auto_mirrored=False
        if n.tag=='Spinner':
            mode=a.get('spinnerMode','dropdown')
            if mode not in {'dropdown','dialog'}:raise ValueError('Unsupported Spinner mode '+mode)
            if mode=='dialog':relative.append('RelativeRule { kind:83, anchor:"dialog" }')
            if 'prompt' in a:relative.append('RelativeRule { kind:84, anchor:'+json.dumps(a['prompt'],ensure_ascii=False)+' }')
            if 'dropDownWidth' in a:
                width=a['dropDownWidth']
                if width not in {'match_parent','wrap_content'}:width=number(width)
                relative.append('RelativeRule { kind:85, anchor:'+json.dumps(width)+' }')
            for name,kind in [('dropDownHorizontalOffset',86),('dropDownVerticalOffset',87)]:
                if name in a:relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(margin_number(a[name]))+' }')
        if n.tag in {'Spinner','ListView'} and 'entries' in a:
                ref=a['entries'];match=re.fullmatch(r'@array/([a-z][a-z0-9_]*)',ref)
                if not match or match.group(1) not in arrays:raise ValueError('Unresolved Spinner entries '+ref)
                values=arrays[match.group(1)]
                if len(values)>256:raise ValueError('Spinner entries exceed limit')
                for value in values:relative.append('RelativeRule { kind:82, anchor:'+json.dumps(resolve(value),ensure_ascii=False)+' }')
        if n.tag not in {'Spinner','ListView'} and 'entries' in a:raise ValueError('entries unsupported on '+n.tag)
        if n.tag not in {'Spinner','AutoCompleteTextView','MultiAutoCompleteTextView'} and any(name in a for name in ATTRS_EXTRA_SPINNER-{'entries'}):raise ValueError('Spinner attribute on unsupported view')
        if n.tag in {'AutoCompleteTextView','MultiAutoCompleteTextView'}:
            if 'spinnerMode' in a or 'prompt' in a:raise ValueError('Spinner attribute on unsupported view')
            threshold=a.get('completionThreshold','2')
            if not threshold.isdigit() or int(threshold)>2048:raise ValueError('Invalid completionThreshold '+threshold)
            relative.append('RelativeRule { kind:97, anchor:'+json.dumps(threshold)+' }')
            for name,kind in [('dropDownWidth',85),('dropDownHeight',98)]:
                if name in a:
                    value=a[name]
                    if value not in {'match_parent','wrap_content'}:value=number(value)
                    relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(value)+' }')
            for name,kind in [('dropDownHorizontalOffset',86),('dropDownVerticalOffset',87)]:
                if name in a:relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(margin_number(a[name]))+' }')
        elif any(name in a for name in ATTRS_EXTRA_AUTO):raise ValueError('AutoComplete attribute on unsupported view')
        if n.tag=='SearchView':
            iconified=a.get('iconifiedByDefault','true')
            if iconified not in {'true','false'}:raise ValueError('Invalid iconifiedByDefault')
            relative.append('RelativeRule { kind:106, anchor:'+json.dumps(iconified)+' }')
            if 'queryHint' in a:relative.append('RelativeRule { kind:107, anchor:'+json.dumps(a['queryHint'],ensure_ascii=False)+' }')
        elif any(name in a for name in ATTRS_EXTRA_SEARCH):raise ValueError('SearchView attribute on unsupported view')
        if n.tag=='TimePicker':
            if a.get('timePickerMode') not in {'spinner','clock'}:raise ValueError('TimePicker must explicitly select clock or spinner mode')
            relative.append('RelativeRule { kind:105, anchor:'+json.dumps(a['timePickerMode'])+' }')
        elif any(name in a for name in ATTRS_EXTRA_TIME):raise ValueError('TimePicker attribute on unsupported view')
        if n.tag in {'CalendarView','DatePicker'}:
            if 'datePickerMode' in a and (n.tag!='DatePicker' or a['datePickerMode'] not in {'calendar','spinner'}):raise ValueError('Invalid DatePicker mode')
            if n.tag=='DatePicker':
                if 'datePickerMode' not in a:raise ValueError('DatePicker must explicitly select a mode')
                relative.append('RelativeRule { kind:102, anchor:'+json.dumps(a['datePickerMode'])+' }')
                for name,kind in [('spinnersShown',103),('calendarViewShown',104)]:
                    if name in a:
                        if a['datePickerMode']!='spinner':raise ValueError(name+' requires DatePicker spinner mode')
                        if a[name] not in {'true','false'}:raise ValueError('Invalid '+name)
                        relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(a[name])+' }')
            elif any(name in a for name in {'spinnersShown','calendarViewShown'}):raise ValueError('DatePicker visibility attribute on unsupported view')
            dates=[]
            for name,kind,default in [('minDate',99,'01/01/1900'),('maxDate',100,'12/31/2100')]:
                value=a.get(name,default)
                try:
                    if not re.fullmatch(r'\d{2}/\d{2}/\d{4}',value):raise ValueError()
                    parsed=datetime.datetime.strptime(value,'%m/%d/%Y').date()
                except ValueError:raise ValueError('Invalid CalendarView '+name+' '+value)
                dates.append(parsed);relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(parsed.isoformat())+' }')
            if dates[0]>dates[1]:raise ValueError('CalendarView maxDate must be >= minDate')
            if 'firstDayOfWeek' in a:
                weekday=a['firstDayOfWeek']
                if weekday not in set('1234567'):raise ValueError('Invalid firstDayOfWeek '+weekday)
                relative.append('RelativeRule { kind:101, anchor:'+json.dumps(weekday)+' }')
        elif any(name in a for name in ATTRS_EXTRA_CALENDAR):raise ValueError('CalendarView attribute on unsupported view')
        if n.tag=='Toolbar':
            if 'navigationContentDescription' in a:relative.append('RelativeRule { kind:111, anchor:'+json.dumps(a['navigationContentDescription'],ensure_ascii=False)+' }')
            for name,kind in [('contentInsetStart',112),('contentInsetEnd',113),('contentInsetLeft',114),('contentInsetRight',115),('contentInsetStartWithNavigation',116),('contentInsetEndWithActions',117),('titleMargin',118),('titleMarginStart',119),('titleMarginEnd',120),('titleMarginTop',121),('titleMarginBottom',122)]:
                if name in a:relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(number(a[name]))+' }')
            for name,kind in [('titleTextColor',123),('subtitleTextColor',124)]:
                if name in a:relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(color(a[name])[len('Some(Color('):-2])+' }')
            for appearance,size_kind,weight_kind,color_kind in [('titleTextAppearance',125,126,129),('subtitleTextAppearance',127,128,130)]:
                if appearance not in a:continue
                values=style_attributes(a[appearance]);allowed={A+'textSize',A+'textStyle',A+'textColor',A+'textAllCaps',A+'letterSpacing',A+'fontFamily',APP+'fontFamily',A+'typeface',A+'textColorHint',A+'textColorLink',A+'lineSpacingMultiplier',A+'lineHeight',APP+'lineHeight',A+'breakStrategy',A+'hyphenationFrequency',A+'justificationMode',A+'includeFontPadding'}
                if set(values)-allowed:raise ValueError('Unsupported Toolbar text appearance attribute')
                if A+'textSize' in values:relative.append('RelativeRule { kind:'+str(size_kind)+', anchor:'+json.dumps(number(values[A+'textSize']))+' }')
                tb_weight=None
                if A+'textStyle' in values:
                    tb_weight='bold' if parse_text_style(resolve(values[A+'textStyle']),'Unsupported Toolbar textStyle ') else 'normal'
                tb_font=resolve(values.get(A+'fontFamily',values.get(APP+'fontFamily',''))).strip()
                if tb_weight!='bold' and tb_font in {'sans-serif-medium','sans-serif-black'}:
                    tb_weight='bold'
                if tb_weight is not None:
                    relative.append('RelativeRule { kind:'+str(weight_kind)+', anchor:'+json.dumps(tb_weight)+' }')
                if A+'textColor' in values:relative.append('RelativeRule { kind:'+str(color_kind)+', anchor:'+json.dumps(color(values[A+'textColor'])[len('Some(Color('):-2])+' }')
        elif any(name in a for name in ATTRS_EXTRA_TOOLBAR):raise ValueError('Toolbar attribute on unsupported view')
        if n.tag in {'ListView','GridView','ExpandableListView'}:
            choice=a.get('choiceMode','none')
            if choice not in {'none','singleChoice','multipleChoice'}:raise ValueError('Unsupported '+n.tag+' choiceMode '+choice)
            relative.append('RelativeRule { kind:88, anchor:'+json.dumps(choice)+' }')
            if n.tag=='GridView' and 'dividerHeight' in a:raise ValueError('ListView attribute on unsupported view')
            for name,kind in [('headerDividersEnabled',89),('footerDividersEnabled',90)]:
                if n.tag not in {'ListView','ExpandableListView'} and name in a:raise ValueError('ListView attribute on unsupported view')
                if name in a:
                    if a[name] not in {'true','false'}:raise ValueError('Invalid '+name)
                    relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(a[name])+' }')
        elif any(name in a for name in ATTRS_EXTRA_LIST):raise ValueError('ListView attribute on unsupported view')
        if 'childDivider' in a:
            if n.tag!='ExpandableListView':raise ValueError('childDivider unsupported on '+n.tag)
            child_color,_=divider_drawable(a['childDivider'],False)
            relative.append('RelativeRule { kind:110, anchor:'+json.dumps(child_color)+' }')
        for name,kind in [('groupIndicator',131),('childIndicator',132)]:
            if name in a:
                if n.tag!='ExpandableListView':raise ValueError(name+' unsupported on '+n.tag)
                anim_hit=resolve_animation_list_drawable(a[name]) if name=='groupIndicator' else None
                if anim_hit is not None:
                    anim_root,anim_ref,anim_base_key,anim_seen=anim_hit
                    anim_spec,first_key,_,any_am=decode_animation_list_frames(anim_root,anim_ref,anim_base_key,anim_seen)
                    if any_am:compound_auto_mirrored=True
                    relative.append('RelativeRule { kind:131, anchor:'+json.dumps('0,0,'+first_key)+' }')
                    relative.append('RelativeRule { kind:295, anchor:'+json.dumps('indicator:'+anim_spec)+' }')
                else:
                    for entry in indicator_resources(a[name],name=='groupIndicator'):relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(entry)+' }')
        for name,kind in [('indicatorLeft',133),('indicatorRight',134),('indicatorStart',135),('indicatorEnd',136),('childIndicatorLeft',137),('childIndicatorRight',138),('childIndicatorStart',139),('childIndicatorEnd',140)]:
            if name in a:
                if n.tag!='ExpandableListView':raise ValueError(name+' unsupported on '+n.tag)
                relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(number(a[name]))+' }')
        if n.tag=='GridView':
            columns=a.get('numColumns','1')
            if columns!='auto_fit' and (not columns.isdigit() or not 1<=int(columns)<=256):raise ValueError('Invalid GridView numColumns '+columns)
            relative.append('RelativeRule { kind:91, anchor:'+json.dumps(columns)+' }')
            for name,kind in [('columnWidth',92),('horizontalSpacing',93),('verticalSpacing',94)]:
                if name in a:relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(number(a[name]))+' }')
            stretch=a.get('stretchMode','columnWidth')
            if stretch not in {'none','spacingWidth','columnWidth','spacingWidthUniform'}:raise ValueError('Unsupported GridView stretchMode '+stretch)
            relative.append('RelativeRule { kind:95, anchor:'+json.dumps(stretch)+' }')
        elif any(name in a for name in ATTRS_EXTRA_GRID):raise ValueError('GridView attribute on unsupported view')
        if material_card:
            if a.get('checkable','false') not in {'true','false'}:raise ValueError('Invalid checkable')
            relative.append('RelativeRule { kind:57, anchor:'+json.dumps(n.attrib.get(APP+'bhooCardVariant','Outlined'))+' }')
            if a.get('checkable')=='true':relative.append('RelativeRule { kind:58, anchor:"" }')
            for name,kind in [('checkedIconSize',59),('checkedIconMargin',60)]:
                if APP+name in n.attrib:relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(number(n.attrib[APP+name]))+' }')
            if APP+'checkedIconGravity' in n.attrib:
                gravity=resolve(n.attrib[APP+'checkedIconGravity'])
                if gravity not in {'TOP_START','TOP_END','BOTTOM_START','BOTTOM_END'}:raise ValueError('Unsupported checkedIconGravity '+gravity)
                relative.append('RelativeRule { kind:61, anchor:'+json.dumps(gravity)+' }')
            for name,kind,selector_kind in [('checkedIconTint',62,75),('cardBackgroundColor',63,76),('strokeColor',64,77),('cardForegroundColor',78,79),('rippleColor',80,81)]:
                if APP+name in n.attrib:
                    raw=n.attrib[APP+name]
                    selector=tint_selector(resolve(raw),True)
                    if selector is not None:
                        for entry in selector:relative.append('RelativeRule { kind:'+str(selector_kind)+', anchor:'+json.dumps(entry)+' }')
                    else:
                        raw=resolve(raw)
                        if raw.startswith('?attr/'):
                            token=raw.removeprefix('?attr/')
                            if token not in CARD_THEME_COLORS:raise ValueError('Unsupported card theme color '+raw)
                            value='@'+token
                        else:value=color(raw).removeprefix('Some(Color(').removesuffix('))')
                        relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(value)+' }')
            for name,kind in [('strokeWidth',65),('cardCornerRadius',66),('cardElevation',71),('cardMaxElevation',72)]:
                if APP+name in n.attrib:relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(number(n.attrib[APP+name]))+' }')
            for name,kind in [('cardUseCompatPadding',73),('cardPreventCornerOverlap',74)]:
                if APP+name in n.attrib:
                    value=resolve(n.attrib[APP+name])
                    if value not in {'true','false'}:raise ValueError('Invalid '+name+' '+value)
                    relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(value)+' }')
            common=n.attrib.get(APP+'contentPadding','0dp')
            for name,kind in [('contentPaddingLeft',67),('contentPaddingTop',68),('contentPaddingRight',69),('contentPaddingBottom',70)]:
                relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(number(n.attrib.get(APP+name,common)))+' }')
        if material_divider:
            for name,kind in [('dividerInsetStart',55),('dividerInsetEnd',56)]:
                if APP+name in n.attrib:
                    relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(number(n.attrib[APP+name]))+' }')
        for name,kind in RELATIVE_ANCHORS.items():
            if name in a:
                if parent not in {'RelativeLayout','TwoLineListItem','DialerFilter'}:
                    if n in included_roots and name not in include_override_attrs.get(n,()):continue
                    raise ValueError('RelativeLayout rule outside RelativeLayout: '+name)
                key=id_key(a[name])
                if not key:raise ValueError('Invalid RelativeLayout anchor '+a[name])
                relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(key)+' }')
        for name,kind in RELATIVE_FLAGS.items():
            if name in a:
                if parent not in {'RelativeLayout','TwoLineListItem','DialerFilter'}:
                    if n in included_roots and name not in include_override_attrs.get(n,()):continue
                    raise ValueError('RelativeLayout rule outside RelativeLayout: '+name)
                if a[name] not in {'true','false'}:raise ValueError('Invalid RelativeLayout boolean '+a[name])
                if a[name]=='true':relative.append('RelativeRule { kind:'+str(kind)+', anchor:"" }')
        if 'ignoreGravity' in a:
            if n.tag not in {'RelativeLayout','TwoLineListItem'}:raise ValueError('ignoreGravity unsupported on '+n.tag)
            key=id_key(a['ignoreGravity'])
            if not key:raise ValueError('Invalid ignoreGravity '+a['ignoreGravity'])
            relative.append('RelativeRule { kind:23, anchor:'+json.dumps(key)+' }')
        for name,kind,minimum in [('layout_column',30,0),('layout_span',31,1)]:
            if name in a:
                if parent=='GridLayout' and name=='layout_column':kind=42
                elif parent!='TableRow':raise ValueError(name+' outside TableRow')
                if not re.fullmatch(r'\d+',a[name]) or not minimum<=int(a[name])<=128:raise ValueError('Invalid '+name+' '+a[name])
                relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(a[name])+' }')
        if 'layout_row' in a:
            if parent!='GridLayout':raise ValueError('layout_row outside GridLayout')
            if not re.fullmatch(r'\d+',a['layout_row']) or int(a['layout_row'])>128:raise ValueError('Invalid layout_row '+a['layout_row'])
            relative.append('RelativeRule { kind:43, anchor:'+json.dumps(a['layout_row'])+' }')
        for name,kind in [('stretchColumns',32),('shrinkColumns',33),('collapseColumns',34)]:
            if name in a:
                if n.tag!='TableLayout':raise ValueError(name+' outside TableLayout')
                value=a[name].strip()
                if value=='*' and name=='collapseColumns':raise ValueError('Invalid '+name+' '+value)
                if value!='*':
                    parts=value.split(',')
                    if not parts or any(not re.fullmatch(r'\s*\d+\s*',p) or int(p)>128 for p in parts):raise ValueError('Invalid '+name+' '+value)
                    value=','.join(str(p) for p in sorted(set(int(p) for p in parts)))
                relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(value)+' }')
        if parent=='TableRow' and any(part in {'start','end','left','right','center_horizontal','center'} for part in a.get('layout_gravity','').split('|')):
            relative.append('RelativeRule { kind:35, anchor:"" }')
        if 'baselineAligned' in a:
            if n.tag not in {'TableRow','LinearLayout','RadioGroup'}:raise ValueError('baselineAligned unsupported on '+n.tag)
            if a['baselineAligned'] not in {'true','false'}:raise ValueError('Invalid baselineAligned '+a['baselineAligned'])
            if a['baselineAligned']=='false':relative.append('RelativeRule { kind:36, anchor:"" }')
        if 'baselineAlignedChildIndex' in a:
            if n.tag not in {'TableRow','LinearLayout','RadioGroup'}:raise ValueError('baselineAlignedChildIndex unsupported on '+n.tag)
            value=a['baselineAlignedChildIndex']
            if not re.fullmatch(r'-1|\d+',value):raise ValueError('Invalid baselineAlignedChildIndex '+value)
            if value!='-1':relative.append('RelativeRule { kind:148, anchor:'+json.dumps(str(int(value)))+' }')
        for name,kind in [('columnCount',40),('rowCount',41)]:
            if name in a:
                if n.tag!='GridLayout':raise ValueError(name+' outside GridLayout')
                if not re.fullmatch(r'\d+',a[name]) or not 1<=int(a[name])<=128:raise ValueError('Invalid '+name+' '+a[name])
                relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(a[name])+' }')
        if parent=='GridLayout' and 'layout_weight' in a:raise ValueError('GridLayout layout_weight is not supported')
        for name,kind in [('layout_columnSpan',44),('layout_rowSpan',45)]:
            if name in a:
                if parent!='GridLayout':raise ValueError(name+' outside GridLayout')
                if not re.fullmatch(r'\d+',a[name]) or not 1<=int(a[name])<=128:raise ValueError('Invalid '+name+' '+a[name])
                relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(a[name])+' }')
        for name,kind in [('layout_columnWeight',46),('layout_rowWeight',47)]:
            if name in a:
                if parent!='GridLayout':raise ValueError(name+' outside GridLayout')
                if not re.fullmatch(r'(?:\d+(?:\.\d+)?|\.\d+)',a[name]) or float(a[name])>10000:raise ValueError('Invalid '+name+' '+a[name])
                relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(a[name])+' }')
        if parent=='GridLayout' and 'layout_gravity' in a:
            parts=a['layout_gravity'].split('|')
            if any(v in {'start','end','left','right','center_horizontal','fill_horizontal','center','fill'} for v in parts):relative.append('RelativeRule { kind:48, anchor:"" }')
            if any(v in {'top','bottom','center_vertical','fill_vertical','center','fill','baseline'} for v in parts):relative.append('RelativeRule { kind:49, anchor:"" }')
        if 'alignmentMode' in a:
            if n.tag!='GridLayout':raise ValueError('alignmentMode outside GridLayout')
            if a['alignmentMode'] not in {'alignBounds','alignMargins'}:raise ValueError('Invalid alignmentMode '+a['alignmentMode'])
            if a['alignmentMode']=='alignBounds':relative.append('RelativeRule { kind:54, anchor:"" }')
        if n.tag=='GridLayout' and 'useDefaultMargins' in a:
            if a['useDefaultMargins'] not in {'true','false'}:raise ValueError('Invalid useDefaultMargins '+a['useDefaultMargins'])
            if a['useDefaultMargins']=='true':relative.append('RelativeRule { kind:53, anchor:"" }')
        elif 'useDefaultMargins' in a:raise ValueError('useDefaultMargins outside GridLayout')
        if parent=='GridLayout':
            all_edges='layout_margin' in a
            mask=sum((1<<i) for i,keys in enumerate([('layout_marginLeft','layout_marginHorizontal'),('layout_marginTop','layout_marginVertical'),('layout_marginRight','layout_marginHorizontal'),('layout_marginBottom','layout_marginVertical')]) if all_edges or any(k in a for k in keys))
            relative.append('RelativeRule { kind:50, anchor:'+json.dumps(str(mask))+' }')
        background_selector=tint_selector(a['background'],compact=True) if 'background' in a and a['background'].startswith('@color/') else None
        if background_selector:
            if any(',@' in rule for rule in background_selector):raise ValueError('Theme attributes are not supported in background selectors')
            relative.extend('RelativeRule { kind:96, anchor:'+json.dumps(rule)+' }' for rule in background_selector)
        if 'loopViews' in a:
            if n.tag not in {'StackView','AdapterViewFlipper'}:raise ValueError('loopViews unsupported on '+n.tag)
            if a['loopViews'] not in {'true','false'}:raise ValueError('Invalid loopViews '+a['loopViews'])
            relative.append('RelativeRule { kind:137, anchor:'+json.dumps(a['loopViews'])+' }')
        for attr,kind in [('tabStripLeft',191),('tabStripRight',192)]:
            if attr not in a:continue
            if n.tag!='TabWidget':raise ValueError(attr+' unsupported on '+n.tag)
            ref=a[attr]
            if ref=='@null':value='null'
            else:
                if not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',ref) or not resource_roots:raise ValueError(attr+' needs a local solid shape drawable')
                path=find_drawable_xml(ref.split('/')[-1])
                if path is None:raise ValueError('Unresolved '+attr+' '+ref)
                shape=ET.parse(path).getroot();solid=shape.find('solid');size_node=shape.find('size')
                if shape.tag!='shape' or shape.attrib.get(A+'shape','rectangle')!='rectangle' or set(shape.attrib)-{A+'shape'} or any(child.tag not in {'solid','size'} or len(child) for child in shape) or len(shape)!=2 or solid is None or size_node is None or set(solid.attrib)!={A+'color'} or set(size_node.attrib)!={A+'width',A+'height'}:raise ValueError(attr+' needs a sized solid rectangle')
                w=float(number(size_node.attrib[A+'width']));h=float(number(size_node.attrib[A+'height']))
                if not 0<w<=4096 or not 0<h<=4096:raise ValueError('Invalid '+attr+' size')
                rgba=color(solid.attrib[A+'color']).removeprefix('Some(Color(').removesuffix('))')
                value=str(w)+','+str(h)+','+rgba
            relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(value)+' }')
        if 'tabStripEnabled' in a:
            if n.tag!='TabWidget':raise ValueError('tabStripEnabled unsupported on '+n.tag)
            if a['tabStripEnabled'] not in {'true','false'}:raise ValueError('Invalid tabStripEnabled')
            relative.append('RelativeRule { kind:147, anchor:'+json.dumps(a['tabStripEnabled'])+' }')
        gallery_attrs={'spacing','unselectedAlpha','animationDuration'}
        if any(name in a for name in gallery_attrs) and n.tag!='Gallery':raise ValueError('Gallery attributes unsupported on '+n.tag)
        if n.tag=='Gallery':
            if 'spacing' in a:relative.append('RelativeRule { kind:144, anchor:'+json.dumps(str(margin_number(a['spacing'])))+' }')
            if 'unselectedAlpha' in a:
                try: alpha=float(a['unselectedAlpha'])
                except ValueError: raise ValueError('Invalid unselectedAlpha')
                if not 0<=alpha<=1:raise ValueError('Invalid unselectedAlpha')
                relative.append('RelativeRule { kind:145, anchor:'+json.dumps(str(alpha))+' }')
            if 'animationDuration' in a:
                try: duration=int(a['animationDuration'])
                except ValueError: raise ValueError('Invalid animationDuration')
                relative.append('RelativeRule { kind:146, anchor:'+json.dumps(str(duration))+' }')
        if n.tag=='DialerFilter':
            children=[child for child in n.iter() if child is not n]
            for required in ('android:hint','android:primary'):
                matches=[child for child in children if id_key(child.attrib.get(A+'id',''))==required]
                if len(matches)!=1 or matches[0].tag!='EditText':raise ValueError('DialerFilter requires EditText '+required)
        drawer_attrs={'handle','content','topOffset','bottomOffset','allowSingleTap','animateOnClick'}
        if n.tag=='SlidingDrawer':
            names=[id_key(a.get(key,'')) for key in ('handle','content')]
            child_ids=[id_key(child.attrib.get(A+'id','')) for child in n.iter() if child is not n]
            if not all(names) or names[0]==names[1] or any(child_ids.count(name)!=1 for name in names):raise ValueError('SlidingDrawer requires distinct, uniquely identified handle and content descendants')
            for name,kind in zip(names,(138,139)):relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(name)+' }')
            for name,kind in [('topOffset',140),('bottomOffset',141)]:
                if name in a:relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(margin_number(a[name]))+' }')
            for name,kind in [('allowSingleTap',142),('animateOnClick',143)]:
                if name in a:
                    if a[name] not in {'true','false'}:raise ValueError('Invalid '+name+' '+a[name])
                    relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(a[name])+' }')
        elif drawer_attrs.intersection(a):raise ValueError('SlidingDrawer attribute on unsupported view')
        if ({'checkMark','checkMarkTint','checkMarkGravity','checkMarkTintMode'} & a.keys()) and n.tag!='CheckedTextView':raise ValueError('checkMark attributes supported only on CheckedTextView')
        if 'button' in a and n.tag not in {'CheckBox','RadioButton'}:raise ValueError('button drawable supported only on CheckBox and RadioButton')
        if 'thumb' in a and n.tag!='SeekBar':raise ValueError('thumb drawable supported only on SeekBar')
        compound={'drawableLeft':169,'drawableTop':170,'drawableRight':171,'drawableBottom':172,'drawableStart':173,'drawableEnd':174}
        if (compound.keys() | {'drawablePadding'}) & a.keys() and n.tag not in {'TextView','CheckedTextView','Button'}:raise ValueError('compound drawables supported only on TextView, CheckedTextView and Button')
        if 'drawableTintMode' in a:
            if n.tag not in {'TextView','CheckedTextView','Button'}:raise ValueError('drawableTintMode supported only on TextView, CheckedTextView and Button')
            modes={'src_atop':0,'src_in':1,'src_over':2,'multiply':3,'screen':4,'add':5}
            mode=a['drawableTintMode']
            if mode not in modes:raise ValueError('Unsupported drawableTintMode '+mode)
            relative.append('RelativeRule { kind:177, anchor:'+json.dumps(str(modes[mode]))+' }')
        if 'drawableTint' in a:
            if n.tag not in {'TextView','CheckedTextView','Button'}:raise ValueError('drawableTint supported only on TextView, CheckedTextView and Button')
            raw=resolve(a['drawableTint']);rules=tint_selector(raw,compact=True)
            if rules is None:rules=['0,0,'+color(raw)[len('Some(Color('):-2]]
            for rule in rules:
                parts=rule.split(',')
                if len(parts)!=6 or (int(parts[0]) | int(parts[1])) & ~(127 if n.tag=='CheckedTextView' else 63):raise ValueError('Unsupported drawableTint selector')
                relative.append('RelativeRule { kind:176, anchor:'+json.dumps(rule)+' }')
        def compound_shape(root):
            shape=root.attrib.get(A+'shape','rectangle')
            if root.tag!='shape' or set(root.attrib)-{A+'shape'} or shape not in {'rectangle','oval'} or any(c.tag not in {'solid','size','corners'} or len(c) for c in root):raise ValueError('Unsupported compound drawable shape')
            solid=root.find('solid');size_node=root.find('size');corners=root.find('corners')
            if solid is None or set(solid.attrib)!={A+'color'} or size_node is None or set(size_node.attrib)!={A+'width',A+'height'} or len(root)!=(2+(corners is not None)):raise ValueError('compound drawable needs solid and size')
            if corners is not None and (shape!='rectangle' or set(corners.attrib)!={A+'radius'}):raise ValueError('Unsupported compound drawable corners')
            width=float(number(size_node.attrib[A+'width']));height=float(number(size_node.attrib[A+'height']))
            if not 0<width<=128 or not 0<height<=128:raise ValueError('Unsupported compound drawable dimensions')
            radius=-1.0 if shape=='oval' else float(number(corners.attrib[A+'radius'])) if corners is not None else 0.0
            return ','.join(map(str,[width,height,radius]))+','+color(solid.attrib[A+'color'])[len('Some(Color('):-2]
        def compound_resource(ref, seen=()):
            if ref=='@null':return '0,0,0,0,0,0,0'
            if ref in seen:raise ValueError('Cyclic drawable reference '+ref)
            tok=theme_token(ref)
            if tok=='listChoiceIndicatorSingle':return '20.0,20.0,-1.0,'+rgba_to_csv(static_color_rgba('?attr/colorControlNormal'))
            if tok=='listChoiceIndicatorMultiple':return '20.0,20.0,2.0,'+rgba_to_csv(static_color_rgba('?attr/colorControlNormal'))
            if not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',ref) or not resource_roots:raise ValueError('compound drawable needs local shape')
            dname=ref.split('/')[-1]
            cands,nodpi,path=find_drawable_candidates(dname)
            if path is not None:
                def compound_element(el, src_ref, elem_name, cur_seen):
                    if el.tag=='shape':return compound_shape(el)
                    if el.tag=='vector':
                        vw,vh,vtint,vfill,_=resolve_and_rasterize_vector(path if el is croot else el,elem_name)
                        rep=vtint or vfill or (0,0,0,255)
                        return ','.join(map(str,[vw,vh,0.0]))+','+rgba_to_csv(rep)
                    if el.tag=='bitmap' and A+'src' in el.attrib:
                        return compound_resource(resolve(el.attrib[A+'src']),cur_seen+(src_ref,))
                    if el.tag=='inset':
                        ia=strip_tools(el.attrib)
                        if A+'drawable' in ia and not len(el):return compound_resource(resolve(ia[A+'drawable']),cur_seen+(src_ref,))
                        if A+'drawable' not in ia and len(el)==1:return compound_element(el[0],src_ref,elem_name,cur_seen)
                        raise ValueError('Invalid inset drawable: '+src_ref)
                    if el.tag in {'rotate','scale','clip'}:
                        kind,payload,_=validate_drawable_wrapper(el,src_ref)
                        return compound_resource(payload,cur_seen+(src_ref,)) if kind=='ref' else compound_element(payload,src_ref,elem_name,cur_seen)
                    if el.tag=='animation-list':
                        chosen=validate_animation_list(el,src_ref)
                        cia=strip_tools(chosen.attrib)
                        if A+'drawable' in cia and not len(chosen):return compound_resource(resolve(cia[A+'drawable']),cur_seen+(src_ref,))
                        if A+'drawable' not in cia and len(chosen)==1:return compound_element(chosen[0],src_ref,elem_name,cur_seen)
                        raise ValueError('Invalid animation-list item: '+src_ref)
                    if el.tag in {'level-list','transition'}:
                        kind,payload,_=select_list_drawable_item(el,src_ref)
                        return compound_resource(payload,cur_seen+(src_ref,)) if kind=='ref' else compound_element(payload,src_ref,elem_name,cur_seen)
                    raise ValueError('Unsupported compound drawable shape')
                croot=ET.parse(path).getroot()
                return compound_element(croot,ref,dname,seen)
            if cands or nodpi is not None:
                if len({dpi for dpi,_ in cands})!=len(cands):raise ValueError('Ambiguous PNG drawable density '+ref)
                if cands:
                    higher=sorted((dpi,f) for dpi,f in cands if dpi>=density)
                    sdpi,img_file=higher[0] if higher else max(cands)
                else:
                    sdpi,img_file=160,nodpi
                if dname not in images:
                    w,h,pixels,scale,_,_=decode_drawable_image(img_file,sdpi)
                    images[dname]=(w,h,pixels,scale)
                iw,ih,_,iscale=images[dname]
                return ','.join(map(str,[round(iw*iscale,4),round(ih*iscale,4),0.0,0,0,0,255]))
            raise ValueError('Unresolved compound drawable '+ref)
        compound_slots={'drawableLeft':'left','drawableTop':'top','drawableRight':'right','drawableBottom':'bottom','drawableStart':'start','drawableEnd':'end'}
        for name,kind in compound.items():
            if name not in a:continue
            ref=a[name];rules=[]
            anim_hit=resolve_animation_list_drawable(ref)
            if anim_hit is not None:
                anim_root,anim_ref,anim_base_key,anim_seen=anim_hit
                anim_spec,first_key,_,any_am=decode_animation_list_frames(anim_root,anim_ref,anim_base_key,anim_seen)
                if any_am:compound_auto_mirrored=True
                try:
                    data=compound_resource(ref)
                except ValueError:
                    iw,ih,_,iscale=images[first_key]
                    data=','.join(map(str,[round(iw*iscale,4),round(ih*iscale,4),0.0,0,0,0,255]))
                relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(data)+' }')
                relative.append('RelativeRule { kind:295, anchor:'+json.dumps(compound_slots[name]+':'+anim_spec)+' }')
                continue
            sel_path=find_drawable_xml(ref.split('/')[-1]) if (ref!='@null' and re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',ref) and resource_roots) else None
            if sel_path is not None:
                root=ET.parse(sel_path).getroot()
                if root.tag in {'selector','animated-selector'}:
                    if (root.tag=='selector' and set(root.attrib)-{A+'constantSize'}) or root.attrib.get(A+'constantSize','false') not in {'true','false'} or not len(root):raise ValueError('Invalid compound drawable selector')
                    constant_size=root.attrib.get(A+'constantSize','false')=='true'
                    flags={'state_enabled':1,'state_pressed':2,'state_focused':4,'state_selected':8,'state_activated':16,'state_hovered':32,'state_checked':64}
                    for item in root:
                        if root.tag=='animated-selector' and item.tag=='transition':continue
                        if item.tag!='item':raise ValueError('Invalid compound drawable selector item')
                        req=forbid=0
                        for attr,value in strip_tools(item.attrib).items():
                            if attr==A+'drawable' or (root.tag=='animated-selector' and attr==A+'id'):continue
                            flag=attr[len(A):] if attr.startswith(A) else ''
                            if flag not in flags or (flag=='state_checked' and n.tag!='CheckedTextView') or resolve(value) not in {'true','false'}:raise ValueError('Unsupported compound drawable selector state')
                            if resolve(value)=='true':req|=flags[flag]
                            else:forbid|=flags[flag]
                        if A+'drawable' in item.attrib and not len(item):data=compound_resource(item.attrib[A+'drawable'])
                        elif A+'drawable' not in item.attrib and len(item)==1:data=compound_shape(item[0])
                        else:raise ValueError('Invalid compound drawable selector item')
                        rules.append((req,forbid,data))
            if rules:
                initial=next((data for required,_,data in rules if required==0),None)
                if constant_size:
                    shapes=[data.split(',') for _,_,data in rules if float(data.split(',')[0])>0]
                    if not shapes:raise ValueError('Compound drawable selector needs a non-null shape')
                    w=max(float(shape[0]) for shape in shapes);h=max(float(shape[1]) for shape in shapes)
                    data=','.join(map(str,[w,h]))+',0,0,0,0,0'
                elif initial is None or float(initial.split(',')[0])<=0:
                    raise ValueError('Compound drawable selector needs an initial intrinsic shape')
                else:data=initial
            else:data=compound_resource(ref)
            relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(data)+' }')
            for req,forbid,data in rules:relative.append('RelativeRule { kind:'+str(kind+9)+', anchor:'+json.dumps(str(req)+','+str(forbid)+','+data)+' }')
        if 'drawablePadding' in a:relative.append('RelativeRule { kind:175, anchor:'+json.dumps(number(a['drawablePadding']))+' }')
        if 'checkMark' in a or 'button' in a or 'thumb' in a:
            flags={'state_enabled':1,'state_pressed':2,'state_focused':4,'state_selected':8,'state_activated':16,'state_hovered':32,'state_checked':64}
            def mark_drawable(ref, seen=()):
                if ref=='@null':return '0,0,0,0,0,0,0'
                if ref in seen:raise ValueError('Cyclic drawable reference '+ref)
                tok=theme_token(ref)
                if ('checkMark' in a or 'button' in a) and tok in {'listChoiceIndicatorSingle','listChoiceIndicatorMultiple'}:
                    rad='10.0' if tok=='listChoiceIndicatorSingle' else '2.0'
                    return '20.0,20.0,'+rad+','+rgba_to_csv(static_color_rgba('?attr/colorPrimary'))
                if not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',ref) or not resource_roots:raise ValueError('checkMark needs local drawable')
                path=find_drawable_xml(ref.split('/')[-1])
                if path is None:raise ValueError('Unresolved checkMark '+ref)
                def mark_element(mroot, cur_ref, cur_seen):
                    if mroot.tag=='inset':
                        ia=strip_tools(mroot.attrib)
                        if A+'drawable' in ia and not len(mroot):return mark_drawable(resolve(ia[A+'drawable']),cur_seen+(cur_ref,))
                        if A+'drawable' not in ia and len(mroot)==1:return mark_element(mroot[0],cur_ref,cur_seen)
                        raise ValueError('Invalid inset drawable: '+cur_ref)
                    if mroot.tag in {'rotate','scale','clip'}:
                        kind,payload,_=validate_drawable_wrapper(mroot,cur_ref)
                        return mark_drawable(payload,cur_seen+(cur_ref,)) if kind=='ref' else mark_element(payload,cur_ref,cur_seen)
                    if mroot.tag in {'level-list','transition'}:
                        kind,payload,_=select_list_drawable_item(mroot,cur_ref)
                        return mark_drawable(payload,cur_seen+(cur_ref,)) if kind=='ref' else mark_element(payload,cur_ref,cur_seen)
                    if mroot.tag=='animation-list':
                        chosen=validate_animation_list(mroot,cur_ref)
                        cia=strip_tools(chosen.attrib)
                        if A+'drawable' in cia and not len(chosen):return mark_drawable(resolve(cia[A+'drawable']),cur_seen+(cur_ref,))
                        if A+'drawable' not in cia and len(chosen)==1:return mark_element(chosen[0],cur_ref,cur_seen)
                        raise ValueError('Invalid animation-list item: '+cur_ref)
                    return mark_shape(mroot)
                return mark_element(ET.parse(path).getroot(),ref,seen)
            def mark_shape(root):
                shape=root.attrib.get(A+'shape','rectangle')
                if root.tag!='shape' or set(root.attrib)-{A+'shape'} or shape not in {'rectangle','oval'} or any(c.tag not in {'solid','size','corners'} or len(c) for c in root):raise ValueError('Unsupported checkMark shape')
                solid=root.find('solid');size_node=root.find('size');corners=root.find('corners')
                if solid is None or set(solid.attrib)!={A+'color'} or size_node is None or set(size_node.attrib)!={A+'width',A+'height'} or len(root)!=(2+(corners is not None)):raise ValueError('checkMark shape needs solid and size')
                if corners is not None and (shape!='rectangle' or set(corners.attrib)!={A+'radius'}):raise ValueError('Unsupported checkMark corners')
                width=float(number(size_node.attrib[A+'width']));height=float(number(size_node.attrib[A+'height']))
                if not 0<width<=128 or not 0<height<=128 or (shape=='oval' and width!=height):raise ValueError('Unsupported checkMark dimensions')
                radius=width/2 if shape=='oval' else float(number(corners.attrib[A+'radius'])) if corners is not None else 0
                return ','.join(map(str,[width,height,radius]))+','+color(solid.attrib[A+'color'])[len('Some(Color('):-2]
            ref=a['checkMark'] if 'checkMark' in a else a['button'] if 'button' in a else a['thumb'];rules=[]
            is_std_choice=('checkMark' in a or 'button' in a) and theme_token(ref) in {'listChoiceIndicatorSingle','listChoiceIndicatorMultiple'}
            if not is_std_choice:
                btn_anim_hit=resolve_animation_list_drawable(ref) if 'button' in a else None
                if btn_anim_hit is not None:
                    anim_root,anim_ref,anim_base_key,anim_seen=btn_anim_hit
                    anim_spec,first_key,_,any_am=decode_animation_list_frames(anim_root,anim_ref,anim_base_key,anim_seen)
                    if any_am:compound_auto_mirrored=True
                    try:
                        static_mark=mark_drawable(ref)
                    except ValueError:
                        iw,ih,_,iscale=images[first_key]
                        static_mark=','.join(map(str,[round(iw*iscale,4),round(ih*iscale,4),0,0,0,0,255]))
                    rules=['0,0,'+static_mark]
                    if any(float(rule.split(',')[2])>32 or float(rule.split(',')[3])>32 for rule in rules):raise ValueError('button drawable dimensions exceed 32dp')
                    relative.append('RelativeRule { kind:165, anchor:'+json.dumps(rules[0])+' }')
                    relative.append('RelativeRule { kind:295, anchor:'+json.dumps('button:'+anim_spec)+' }')
                else:
                    mark_path=find_drawable_xml(ref.split('/')[-1]) if (ref!='@null' and re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',ref) and resource_roots) else None
                    if mark_path is not None:
                        root=ET.parse(mark_path).getroot()
                        if root.tag in {'selector','animated-selector'}:
                            if (root.tag=='selector' and root.attrib) or not len(root):raise ValueError('Invalid checkMark selector')
                            for item in root:
                                if root.tag=='animated-selector' and item.tag=='transition':continue
                                if item.tag!='item':raise ValueError('Invalid checkMark selector item')
                                req=forbid=0
                                for attr,value in strip_tools(item.attrib).items():
                                    if attr==A+'drawable' or (root.tag=='animated-selector' and attr==A+'id'):continue
                                    name=attr[len(A):] if attr.startswith(A) else ''
                                    if name not in flags or resolve(value) not in {'true','false'}:raise ValueError('Unsupported checkMark selector state')
                                    if resolve(value)=='true':req|=flags[name]
                                    else:forbid|=flags[name]
                                if A+'drawable' in item.attrib and not len(item):mark=mark_drawable(item.attrib[A+'drawable'])
                                elif A+'drawable' not in item.attrib and len(item)==1:mark=mark_shape(item[0])
                                else:raise ValueError('Invalid checkMark selector drawable')
                                rules.append(str(req)+','+str(forbid)+','+mark)
                    if not rules:rules=['0,0,'+mark_drawable(ref)]
                    if 'button' in a and any(float(rule.split(',')[2])>32 or float(rule.split(',')[3])>32 for rule in rules):raise ValueError('button drawable dimensions exceed 32dp')
                    if 'thumb' in a and any((int(rule.split(',')[0]) | int(rule.split(',')[1])) & ~63 for rule in rules):raise ValueError('Unsupported SeekBar thumb state')
                    for rule in rules:relative.append('RelativeRule { kind:'+('164' if 'checkMark' in a else '165' if 'button' in a else '166')+', anchor:'+json.dumps(rule)+' }')
        if 'buttonTintMode' in a:
            if n.tag not in {'CheckBox','RadioButton'} or 'button' not in a:raise ValueError('buttonTintMode requires a custom CheckBox or RadioButton drawable')
            modes={'src_atop':0,'src_in':1,'src_over':2,'multiply':3,'screen':4,'add':5}
            mode=a['buttonTintMode']
            if mode not in modes:raise ValueError('Unsupported buttonTintMode '+mode)
            relative.append('RelativeRule { kind:184, anchor:'+json.dumps(str(modes[mode]))+' }')
        if 'buttonTint' in a:
            if n.tag not in {'CheckBox','RadioButton'}:raise ValueError('buttonTint supported only on CheckBox and RadioButton')
            raw=resolve(a['buttonTint']);rules=tint_selector(raw,compact=True)
            if rules is None:rules=['0,0,'+color(raw)[len('Some(Color('):-2]]
            for rule in rules:
                parts=rule.split(',')
                if len(parts)!=6 or (int(parts[0]) | int(parts[1])) & ~127:raise ValueError('Unsupported buttonTint selector')
                relative.append('RelativeRule { kind:158, anchor:'+json.dumps(rule)+' }')
        if 'checkMarkGravity' in a:
            if a['checkMarkGravity'] not in {'start','end','left','right'}:raise ValueError('Unsupported checkMarkGravity')
            relative.append('RelativeRule { kind:157, anchor:'+json.dumps(a['checkMarkGravity'])+' }')
        if 'checkMarkTintMode' in a:
            if n.tag!='CheckedTextView' or 'checkMark' not in a:raise ValueError('checkMarkTintMode requires a custom CheckedTextView mark')
            modes={'src_atop':0,'src_in':1,'src_over':2,'multiply':3,'screen':4,'add':5}
            mode=a['checkMarkTintMode']
            if mode not in modes:raise ValueError('Unsupported checkMarkTintMode '+mode)
            relative.append('RelativeRule { kind:185, anchor:'+json.dumps(str(modes[mode]))+' }')
        if 'checkMarkTint' in a:
            raw=resolve(a['checkMarkTint']);rules=tint_selector(raw,compact=True)
            if rules is None:rules=['0,0,'+color(raw)[len('Some(Color('):-2]]
            for rule in rules:
                parts=rule.split(',')
                if len(parts)!=6 or (int(parts[0]) | int(parts[1])) & ~127:raise ValueError('Unsupported checkMarkTint selector')
                relative.append('RelativeRule { kind:156, anchor:'+json.dumps(rule)+' }')
        if 'disabledAlpha' in a:
            if n.tag!='ToggleButton':raise ValueError('disabledAlpha supported only on ToggleButton')
            try: disabled_alpha=float(a['disabledAlpha'])
            except ValueError: raise ValueError('Invalid disabledAlpha')
            if not 0<=disabled_alpha<=1:raise ValueError('Invalid disabledAlpha')
            relative.append('RelativeRule { kind:162, anchor:'+json.dumps(str(disabled_alpha))+' }')
        progress_tints={'progressTint':149,'progressBackgroundTint':150,'secondaryProgressTint':151,'indeterminateTint':152}
        if n.tag not in {'ProgressBar','SeekBar','RatingBar'} and progress_tints.keys() & a.keys():raise ValueError('Progress tint attributes supported only on ProgressBar, SeekBar and RatingBar')
        for attr,drawable,kind in [('thumbTintMode','thumb',186),('tickMarkTintMode','tickMark',187)]:
            if attr in a:
                if n.tag!='SeekBar' or drawable not in a:raise ValueError(attr+' requires a custom SeekBar '+drawable)
                modes={'src_atop':0,'src_in':1,'src_over':2,'multiply':3,'screen':4,'add':5}
                mode=a[attr]
                if mode not in modes:raise ValueError('Unsupported '+attr+' '+mode)
                relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(str(modes[mode]))+' }')
        if 'thumbTint' in a and n.tag not in {'SeekBar','Switch'}:raise ValueError('thumbTint supported only on SeekBar and Switch')
        if 'trackTint' in a and n.tag!='Switch':raise ValueError('trackTint supported only on Switch')
        if 'splitTrack' in a:
            if n.tag!='SeekBar':raise ValueError('splitTrack supported only on SeekBar')
            if a['splitTrack'] not in {'true','false'}:raise ValueError('Invalid splitTrack')
            if a['splitTrack']=='true':relative.append('RelativeRule { kind:163, anchor:"" }')
        if 'showText' in a and n.tag!='Switch':raise ValueError('showText supported only on Switch')
        if 'showText' in a and a['showText'] not in {'true','false'}:raise ValueError('Invalid showText')
        for name,kind in [('switchMinWidth',193),('switchPadding',194),('thumbTextPadding',195)]:
            if name not in a:continue
            if n.tag!='Switch':raise ValueError(name+' supported only on Switch')
            value=float(number(a[name]))
            if not 0<=value<=4096:raise ValueError('Invalid '+name)
            relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(str(value))+' }')
        if n.tag=='Switch' and (a.get('showText')=='true' or ('showText' not in a and ('textOn' in a or 'textOff' in a))):relative.append('RelativeRule { kind:161, anchor:"" }')
        if n.tag=='Switch':
            for name,kind in [('thumbTint',159),('trackTint',160)]:
                if name not in a:continue
                raw=resolve(a[name]);rules=tint_selector(raw,compact=True)
                if rules is None:rules=['0,0,'+color(raw)[len('Some(Color('):-2]]
                for rule in rules:
                    parts=rule.split(',')
                    if len(parts)!=6 or (int(parts[0]) | int(parts[1])) & ~127:raise ValueError('Unsupported Switch tint selector')
                    relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(rule)+' }')
        if ({'tickMark','tickMarkTint'} & a.keys()) and n.tag!='SeekBar':raise ValueError('tickMark supported only on SeekBar')
        if 'tickMarkTint' in a and 'tickMark' not in a:raise ValueError('tickMarkTint requires tickMark')
        if 'tickMark' in a:
            ref=a['tickMark']
            def tick_ref(v, seen=()):
                if v=='@null':return '0,0,0,0,0,0,0'
                if v in seen:raise ValueError('Cyclic drawable reference '+v)
                if not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',v) or not resource_roots:raise ValueError('tickMark needs local shape drawable')
                path=find_drawable_xml(v.split('/')[-1])
                if path is None:raise ValueError('Unresolved tickMark '+v)
                def tick_element(troot, cur_v, cur_seen):
                    if troot.tag=='inset':
                        ia=strip_tools(troot.attrib)
                        if A+'drawable' in ia and not len(troot):return tick_ref(resolve(ia[A+'drawable']),cur_seen+(cur_v,))
                        if A+'drawable' not in ia and len(troot)==1:return tick_element(troot[0],cur_v,cur_seen)
                        raise ValueError('Invalid inset drawable: '+cur_v)
                    if troot.tag in {'rotate','scale','clip'}:
                        kind,payload,_=validate_drawable_wrapper(troot,cur_v)
                        return tick_ref(payload,cur_seen+(cur_v,)) if kind=='ref' else tick_element(payload,cur_v,cur_seen)
                    if troot.tag in {'level-list','transition'}:
                        kind,payload,_=select_list_drawable_item(troot,cur_v)
                        return tick_ref(payload,cur_seen+(cur_v,)) if kind=='ref' else tick_element(payload,cur_v,cur_seen)
                    if troot.tag=='animation-list':
                        chosen=validate_animation_list(troot,cur_v)
                        cia=strip_tools(chosen.attrib)
                        if A+'drawable' in cia and not len(chosen):return tick_ref(resolve(cia[A+'drawable']),cur_seen+(cur_v,))
                        if A+'drawable' not in cia and len(chosen)==1:return tick_element(chosen[0],cur_v,cur_seen)
                        raise ValueError('Invalid animation-list item: '+cur_v)
                    return tick_shape(troot)
                return tick_element(ET.parse(path).getroot(),v,seen)
            def tick_shape(root):
                shape=root.attrib.get(A+'shape','rectangle')
                if root.tag!='shape' or set(root.attrib)-{A+'shape'} or shape not in {'rectangle','oval'} or any(child.tag not in {'solid','size','corners'} for child in root):raise ValueError('Unsupported tickMark shape')
                solid=root.find('solid');size_node=root.find('size');corners=root.find('corners')
                if solid is None or set(solid.attrib)!={A+'color'} or size_node is None or set(size_node.attrib)!={A+'width',A+'height'} or len(root)!=(2+(corners is not None)) or any(len(child) for child in root):raise ValueError('tickMark requires one solid and explicit width and height')
                if corners is not None and (shape!='rectangle' or set(corners.attrib)!={A+'radius'}):raise ValueError('Unsupported tickMark corners')
                width=float(number(size_node.attrib[A+'width']));height=float(number(size_node.attrib[A+'height']))
                if not 0<width<=128 or not 0<height<=128 or (shape=='oval' and width!=height):raise ValueError('Unsupported tickMark dimensions')
                radius=width/2 if shape=='oval' else float(number(corners.attrib[A+'radius'])) if corners is not None else 0
                return ','.join(map(str,[width,height,radius]))+','+color(solid.attrib[A+'color'])[len('Some(Color('):-2]
            tick_anim_hit=resolve_animation_list_drawable(ref)
            if tick_anim_hit is not None:
                anim_root,anim_ref,anim_base_key,anim_seen=tick_anim_hit
                anim_spec,first_key,_,any_am=decode_animation_list_frames(anim_root,anim_ref,anim_base_key,anim_seen)
                if any_am:compound_auto_mirrored=True
                try:
                    static_tick=tick_ref(ref)
                except ValueError:
                    iw,ih,_,iscale=images[first_key]
                    static_tick=','.join(map(str,[round(iw*iscale,4),round(ih*iscale,4),0,0,0,0,255]))
                relative.append('RelativeRule { kind:155, anchor:'+json.dumps(static_tick)+' }')
                relative.append('RelativeRule { kind:295, anchor:'+json.dumps('tick:'+anim_spec)+' }')
            else:
                root=None
                if ref!='@null' and re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',ref) and resource_roots:
                    path=find_drawable_xml(ref.split('/')[-1])
                    if path is not None:root=ET.parse(path).getroot()
                if root is not None and root.tag in {'selector','animated-selector'}:
                    if (root.tag=='selector' and root.attrib) or not len(root):raise ValueError('Invalid tickMark selector')
                    flags={'state_enabled':1,'state_pressed':2,'state_focused':4,'state_selected':8,'state_activated':16,'state_hovered':32}
                    tick_rules=[]
                    for item in root:
                        if root.tag=='animated-selector' and item.tag=='transition':continue
                        if item.tag!='item':raise ValueError('Invalid tickMark selector item')
                        required=forbidden=0
                        for attr,value in strip_tools(item.attrib).items():
                            if attr==A+'drawable' or (root.tag=='animated-selector' and attr==A+'id'):continue
                            name=attr[len(A):] if attr.startswith(A) else ''
                            if name not in flags or resolve(value) not in {'true','false'}:raise ValueError('Unsupported tickMark selector state')
                            if resolve(value)=='true':required|=flags[name]
                            else:forbidden|=flags[name]
                        if A+'drawable' in item.attrib and not len(item):mark=tick_ref(item.attrib[A+'drawable'])
                        elif A+'drawable' not in item.attrib and len(item)==1:mark=tick_shape(item[0])
                        else:raise ValueError('Invalid tickMark selector drawable')
                        tick_rules.append('RelativeRule { kind:167, anchor:'+json.dumps(str(required)+','+str(forbidden)+','+mark)+' }')
                    if not tick_rules:raise ValueError('Invalid tickMark selector')
                    relative.extend(tick_rules)
                elif ref=='@null':relative.append('RelativeRule { kind:167, anchor:"0,0,0,0,0,0,0,0,0" }')
                else:relative.append('RelativeRule { kind:155, anchor:'+json.dumps(tick_ref(ref))+' }')
        if n.tag in {'SeekBar','RatingBar'} and 'indeterminateTint' in a:raise ValueError(n.tag+' indeterminate tint is not supported')
        if n.tag=='SeekBar':progress_tints['thumbTint']=153
        if n.tag=='SeekBar':progress_tints['tickMarkTint']=154
        for name,kind in progress_tints.items():
            if name not in a:continue
            raw=resolve(a[name])
            rules=tint_selector(raw, compact=True)
            if rules is None:rules=['0,0,'+color(raw)[len('Some(Color('):-2]]
            for rule in rules:
                parts=rule.split(',')
                if len(parts)!=6 or (int(parts[0]) | int(parts[1])) & ~63:raise ValueError('Unsupported ProgressBar tint selector')
                relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(rule)+' }')
        if n.tag=='Button' and 'gravity' in a:relative.append('RelativeRule { kind:168, anchor:"" }')
        if 'cropToPadding' in a:
            if n.tag not in {'ImageView','ImageButton','ZoomButton','QuickContactBadge'}:raise ValueError('cropToPadding unsupported on '+n.tag)
            if resolve(a['cropToPadding']) not in {'true','false'}:raise ValueError('Invalid cropToPadding')
            if resolve(a['cropToPadding'])=='true':relative.append('RelativeRule { kind:188, anchor:"" }')
        if {'baseline','baselineAlignBottom'} & a.keys():
            if n.tag not in {'ImageView','ImageButton','ZoomButton','QuickContactBadge'}:raise ValueError('ImageView baseline attributes unsupported on '+n.tag)
            if 'baseline' in a:
                value=resolve(a['baseline'])
                if value!='-1':relative.append('RelativeRule { kind:189, anchor:'+json.dumps(number(value))+' }')
            if 'baselineAlignBottom' in a:
                value=resolve(a['baselineAlignBottom'])
                if value not in {'true','false'}:raise ValueError('Invalid baselineAlignBottom')
                if value=='true':relative.append('RelativeRule { kind:190, anchor:"" }')
        if 'elevation' in a:
            elev=number(a['elevation'])
            if float(elev)>0:relative.append('RelativeRule { kind:193, anchor:'+json.dumps(elev)+' }')
        if 'backgroundTint' in a:
            relative.append('RelativeRule { kind:194, anchor:'+json.dumps(rgba_to_csv(static_color_rgba(a['backgroundTint'])))+' }')
        if 'textAlignment' in a:
            if a['textAlignment'] not in {'textStart','viewStart','center','textEnd','viewEnd','left','right','gravity'}:raise ValueError('Unsupported textAlignment '+a['textAlignment'])
            relative.append('RelativeRule { kind:195, anchor:'+json.dumps(a['textAlignment'])+' }')
        for name,kind in CONSTRAINT_ANCHORS.items():
            if APP+name not in n.attrib:continue
            raw=resolve(n.attrib[APP+name])
            anchor='parent' if raw=='parent' else id_key(raw)
            if anchor is None:raise ValueError('Invalid ConstraintLayout anchor '+raw)
            relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(anchor)+' }')
        constraint_phys_mask=0
        if (APP+'layout_constraintLeft_toLeftOf' in n.attrib or APP+'layout_constraintLeft_toRightOf' in n.attrib) and not (APP+'layout_constraintStart_toStartOf' in n.attrib or APP+'layout_constraintStart_toEndOf' in n.attrib):
            constraint_phys_mask|=1
        if (APP+'layout_constraintRight_toRightOf' in n.attrib or APP+'layout_constraintRight_toLeftOf' in n.attrib) and not (APP+'layout_constraintEnd_toEndOf' in n.attrib or APP+'layout_constraintEnd_toStartOf' in n.attrib):
            constraint_phys_mask|=2
        if constraint_phys_mask:
            relative.append('RelativeRule { kind:287, anchor:'+json.dumps(str(constraint_phys_mask))+' }')
        for name,kind in CONSTRAINT_BIASES.items():
            if APP+name not in n.attrib:continue
            raw=resolve(n.attrib[APP+name])
            try: bias=float(raw)
            except ValueError: raise ValueError('Invalid ConstraintLayout bias '+raw)
            if not 0.0<=bias<=1.0:raise ValueError('Invalid ConstraintLayout bias '+raw)
            relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(str(bias))+' }')
        if n in guideline_specs:
            g_orient,g_begin,g_end,g_pct=guideline_specs[n]
            relative.append('RelativeRule { kind:210, anchor:'+json.dumps(g_orient)+' }')
            if g_begin is not None:relative.append('RelativeRule { kind:211, anchor:'+json.dumps(g_begin)+' }')
            if g_end is not None:relative.append('RelativeRule { kind:212, anchor:'+json.dumps(g_end)+' }')
            if g_pct is not None:relative.append('RelativeRule { kind:213, anchor:'+json.dumps(g_pct)+' }')
        for name,kind in CONSTRAINT_CHAIN_STYLES.items():
            if APP+name not in n.attrib:continue
            raw=resolve(n.attrib[APP+name]).strip()
            mapped={'spread':'spread','spread_inside':'spread_inside','packed':'packed','0':'spread','1':'spread_inside','2':'packed'}.get(raw)
            if mapped is None:raise ValueError('Unsupported ConstraintLayout chainStyle '+raw)
            relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(mapped)+' }')
        for name,kind in CONSTRAINT_WEIGHTS.items():
            if APP+name not in n.attrib:continue
            raw=resolve(n.attrib[APP+name]).strip()
            try: weight=float(raw)
            except ValueError: raise ValueError('Invalid ConstraintLayout weight '+raw)
            if not (weight>=0.0) or weight!=weight or weight==float('inf'):raise ValueError('Invalid ConstraintLayout weight '+raw)
            if weight>0.0:relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(str(weight))+' }')
        if n in barrier_specs:
            relative.append('RelativeRule { kind:224, anchor:'+json.dumps(barrier_specs[n])+' }')
        if n in group_specs:
            relative.append('RelativeRule { kind:225, anchor:'+json.dumps(group_specs[n])+' }')
        if n in flow_specs:
            relative.append('RelativeRule { kind:232, anchor:'+json.dumps(flow_specs[n])+' }')
        if n in flow_ext_specs:
            relative.append('RelativeRule { kind:292, anchor:'+json.dumps(flow_ext_specs[n])+' }')
        if n in layer_specs:
            relative.append('RelativeRule { kind:281, anchor:'+json.dumps(layer_specs[n])+' }')
        if n in placeholder_specs:
            relative.append('RelativeRule { kind:282, anchor:'+json.dumps(placeholder_specs[n])+' }')
        if APP+'layout_constraintCircle' in n.attrib:
            raw_circle=resolve(n.attrib[APP+'layout_constraintCircle']).strip()
            circle_target='parent' if raw_circle=='parent' else id_key(raw_circle)
            if circle_target is None:raise ValueError('Invalid layout_constraintCircle '+raw_circle)
            circle_radius=number(n.attrib[APP+'layout_constraintCircleRadius']) if APP+'layout_constraintCircleRadius' in n.attrib else '0'
            if APP+'layout_constraintCircleAngle' in n.attrib:
                raw_angle=resolve(n.attrib[APP+'layout_constraintCircleAngle']).strip()
                try: circle_angle_v=float(raw_angle)
                except ValueError: raise ValueError('Invalid layout_constraintCircleAngle '+raw_angle)
                if circle_angle_v!=circle_angle_v or circle_angle_v in (float('inf'),float('-inf')):raise ValueError('Invalid layout_constraintCircleAngle '+raw_angle)
                circle_angle=str(circle_angle_v)
            else:
                circle_angle='0'
            relative.append('RelativeRule { kind:233, anchor:'+json.dumps(f'{circle_target},{circle_radius},{circle_angle}')+' }')
        else:
            if APP+'layout_constraintCircleRadius' in n.attrib:number(n.attrib[APP+'layout_constraintCircleRadius'])
            if APP+'layout_constraintCircleAngle' in n.attrib:
                raw_angle=resolve(n.attrib[APP+'layout_constraintCircleAngle']).strip()
                try: circle_angle_v=float(raw_angle)
                except ValueError: raise ValueError('Invalid layout_constraintCircleAngle '+raw_angle)
                if circle_angle_v!=circle_angle_v or circle_angle_v in (float('inf'),float('-inf')):raise ValueError('Invalid layout_constraintCircleAngle '+raw_angle)
        if APP+'layout_constraintDimensionRatio' in n.attrib:
            raw_ratio=resolve(n.attrib[APP+'layout_constraintDimensionRatio']).strip()
            m_ratio=re.fullmatch(r'(?:([WwHh])\s*,\s*)?(.+)',raw_ratio)
            if not m_ratio:raise ValueError('Invalid layout_constraintDimensionRatio '+raw_ratio)
            body=m_ratio.group(2).strip()
            try:
                if ':' in body:
                    p1,p2=body.split(':',1)
                    v1,v2=float(p1.strip()),float(p2.strip())
                    valid_ratio=v1>0 and v2>0 and v1==v1 and v2==v2 and v1!=float('inf') and v2!=float('inf')
                else:
                    v1=float(body)
                    valid_ratio=v1>0 and v1==v1 and v1!=float('inf')
            except ValueError:
                valid_ratio=False
            if not valid_ratio:raise ValueError('Invalid layout_constraintDimensionRatio '+raw_ratio)
            relative.append('RelativeRule { kind:219, anchor:'+json.dumps(raw_ratio)+' }')
        cw_flag=0
        ch_flag=0
        for cname in ('layout_constrainedWidth','layout_constrainedHeight'):
            if APP+cname in n.attrib:
                cv=resolve(n.attrib[APP+cname]).strip()
                if cv not in {'true','false'}:
                    raise ValueError('Invalid '+cname+' '+n.attrib[APP+cname])
                if cname=='layout_constrainedWidth':cw_flag=1 if cv=='true' else 0
                else:ch_flag=1 if cv=='true' else 0
        size_bound_vals=[]
        for bname in ('layout_constraintWidth_min','layout_constraintWidth_max','layout_constraintHeight_min','layout_constraintHeight_max'):
            if APP+bname in n.attrib:
                raw_bv=resolve(n.attrib[APP+bname]).strip()
                if raw_bv=='wrap':
                    bv=0.0
                else:
                    bv=float(number(raw_bv))
                size_bound_vals.append(bv)
            else:
                size_bound_vals.append(0.0)
        if cw_flag or ch_flag or any(bv>0.0 for bv in size_bound_vals):
            b_strs=[fmt_num(bv) for bv in size_bound_vals]
            relative.append('RelativeRule { kind:280, anchor:'+json.dumps(f'{cw_flag},{ch_flag}|'+','.join(b_strs))+' }')
        cw_def='spread'
        ch_def='spread'
        for cname in ('layout_constraintWidth_default','layout_constraintHeight_default'):
            if APP+cname in n.attrib:
                cdef=resolve(n.attrib[APP+cname]).strip()
                if cdef not in {'spread','wrap','percent'}:
                    raise ValueError('Invalid '+cname+' '+n.attrib[APP+cname])
                if cname=='layout_constraintWidth_default':cw_def=cdef
                else:ch_def=cdef
        if cw_def!='spread' or ch_def!='spread':
            relative.append('RelativeRule { kind:294, anchor:'+json.dumps(f'{cw_def},{ch_def}')+' }')
        for cname,kind in (('layout_constraintWidth_percent',220),('layout_constraintHeight_percent',221)):
            if APP+cname in n.attrib:
                raw=resolve(n.attrib[APP+cname])
                try: pct=float(raw)
                except ValueError: raise ValueError('Invalid '+cname+' '+raw)
                if not 0.0<=pct<=1.0:raise ValueError('Invalid '+cname+' '+raw)
                relative.append('RelativeRule { kind:'+str(kind)+', anchor:'+json.dumps(str(pct))+' }')
        gone_keys=('layout_goneMarginLeft','layout_goneMarginTop','layout_goneMarginRight','layout_goneMarginBottom','layout_goneMarginStart','layout_goneMarginEnd')
        has_gone_base=APP+'layout_goneMarginBaseline' in n.attrib
        if any(APP+k in n.attrib for k in gone_keys) or has_gone_base:
            vals=[margin_number(n.attrib[APP+k]) if APP+k in n.attrib else '-1' for k in gone_keys]
            if has_gone_base:
                vals.append(margin_number(n.attrib[APP+'layout_goneMarginBaseline']))
            relative.append('RelativeRule { kind:223, anchor:'+json.dumps(','.join(vals))+' }')
        if APP+'layout_constraintBaseline_toBaselineOf' in n.attrib:
            raw=resolve(n.attrib[APP+'layout_constraintBaseline_toBaselineOf'])
            anchor='parent' if raw=='parent' else id_key(raw)
            if anchor is None:raise ValueError('Invalid ConstraintLayout anchor '+raw)
            relative.append('RelativeRule { kind:222, anchor:'+json.dumps(anchor)+' }')
        if parent_node in coordinator_nodes:
            if APP+'layout_behavior' in n.attrib:
                raw_beh=n.attrib[APP+'layout_behavior'].strip()
                try: resolved_beh=resolve(raw_beh)
                except ValueError: resolved_beh=raw_beh
                if any(tok in s for s in (raw_beh,resolved_beh) for tok in ('appbar_scrolling_view_behavior','ScrollingViewBehavior')):
                    relative.append('RelativeRule { kind:226, anchor:"" }')
                if any(tok in s for s in (raw_beh,resolved_beh) for tok in ('BottomSheetBehavior','bottom_sheet_behavior')):
                    bs_peek='-1'
                    if APP+'behavior_peekHeight' in n.attrib:
                        raw_pk=resolve(n.attrib[APP+'behavior_peekHeight']).strip()
                        if raw_pk in {'auto','-1'}:
                            bs_peek='-1'
                        else:
                            try: bs_peek=number(raw_pk)
                            except ValueError: raise ValueError('Invalid behavior_peekHeight '+raw_pk)
                    bs_hide='0'
                    if APP+'behavior_hideable' in n.attrib:
                        bv=resolve(n.attrib[APP+'behavior_hideable']).strip()
                        if bv not in {'true','false'}:raise ValueError('Invalid behavior_hideable '+bv)
                        bs_hide='1' if bv=='true' else '0'
                    bs_fit='1'
                    if APP+'behavior_fitToContents' in n.attrib:
                        bv=resolve(n.attrib[APP+'behavior_fitToContents']).strip()
                        if bv not in {'true','false'}:raise ValueError('Invalid behavior_fitToContents '+bv)
                        bs_fit='1' if bv=='true' else '0'
                    bs_half='0.5'
                    if APP+'behavior_halfExpandedRatio' in n.attrib:
                        raw_hr=resolve(n.attrib[APP+'behavior_halfExpandedRatio']).strip()
                        try: hr=float(raw_hr)
                        except ValueError: raise ValueError('Invalid behavior_halfExpandedRatio '+raw_hr)
                        if not (0.0<hr<1.0) or hr!=hr:raise ValueError('Invalid behavior_halfExpandedRatio '+raw_hr)
                        bs_half=str(hr)
                    bs_skip='0'
                    if APP+'behavior_skipCollapsed' in n.attrib:
                        bv=resolve(n.attrib[APP+'behavior_skipCollapsed']).strip()
                        if bv not in {'true','false'}:raise ValueError('Invalid behavior_skipCollapsed '+bv)
                        bs_skip='1' if bv=='true' else '0'
                    relative.append('RelativeRule { kind:259, anchor:'+json.dumps(f'{bs_peek}|{bs_hide}|{bs_fit}|{bs_half}|{bs_skip}')+' }')
                if any(tok in s for s in (raw_beh,resolved_beh) for tok in ('SideSheetBehavior','side_sheet_behavior')):
                    ss_grav_parts=[p.strip() for p in a.get('layout_gravity','').split('|') if p.strip()]
                    ss_edge='start' if any(p in {'start','left'} for p in ss_grav_parts) else 'end'
                    ss_coplanar='1' if any('coplanar' in s.lower() for s in node_style_names.get(n,())) else '0'
                    if APP+'coplanarSiblingViewId' in n.attrib:
                        raw_cp=resolve(n.attrib[APP+'coplanarSiblingViewId']).strip()
                        if raw_cp and raw_cp!='@null':
                            if id_key(raw_cp) is None:raise ValueError('Invalid coplanarSiblingViewId '+raw_cp)
                            ss_coplanar='1'
                    relative.append('RelativeRule { kind:261, anchor:'+json.dumps(f'{ss_edge}|{ss_coplanar}')+' }')
            if APP+'layout_anchor' in n.attrib:
                raw_anc=resolve(n.attrib[APP+'layout_anchor']).strip()
                if raw_anc and raw_anc!='@null':
                    target_id=id_key(raw_anc)
                    if target_id is None:raise ValueError('Invalid CoordinatorLayout anchor '+raw_anc)
                    if APP+'layout_anchorGravity' in n.attrib:
                        raw_grav=resolve(n.attrib[APP+'layout_anchorGravity']).strip()
                        tokens=[]
                        for part in raw_grav.split('|'):
                            tok=part.strip()
                            if not tok:continue
                            if tok not in COORDINATOR_ANCHOR_GRAVITIES:raise ValueError('Unsupported layout_anchorGravity '+tok)
                            if tok not in tokens:tokens.append(tok)
                        if not tokens:raise ValueError('Unsupported layout_anchorGravity '+raw_grav)
                        grav_str='|'.join(tokens)
                    else:
                        grav_str='center'
                    relative.append('RelativeRule { kind:227, anchor:'+json.dumps(f'{target_id}:{grav_str}')+' }')
        if APP+'layout_scrollFlags' in n.attrib:
            sf_parts=[p.strip() for p in resolve(n.attrib[APP+'layout_scrollFlags']).split('|') if p.strip()]
            if not sf_parts or any(p not in {'noScroll','scroll','exitUntilCollapsed','enterAlways','enterAlwaysCollapsed','snap','snapMargins'} for p in sf_parts):
                raise ValueError('Unsupported layout_scrollFlags '+n.attrib[APP+'layout_scrollFlags'])
            sf_dedup=[]
            for p in sf_parts:
                if p not in sf_dedup:sf_dedup.append(p)
            relative.append('RelativeRule { kind:284, anchor:'+json.dumps(','.join(sf_dedup))+' }')
        if APP+'layout_scrollEffect' in n.attrib:
            se=resolve(n.attrib[APP+'layout_scrollEffect']).strip()
            if se not in {'none','compress'}:raise ValueError('Unsupported layout_scrollEffect '+se)
            if se=='compress':
                relative.append('RelativeRule { kind:290, anchor:"effect=compress" }')
        if APP+'drawerLockMode' in n.attrib:
            raw_dlm=resolve(n.attrib[APP+'drawerLockMode']).strip()
            dlm={'unlocked':'unlocked','locked_closed':'locked_closed','locked_open':'locked_open','0':'unlocked','1':'locked_closed','2':'locked_open'}.get(raw_dlm)
            if dlm is None:raise ValueError('Unsupported drawerLockMode '+raw_dlm)
            relative.append('RelativeRule { kind:291, anchor:'+json.dumps(dlm)+' }')
        if APP+'behavior_overlapTop' in n.attrib:
            number(n.attrib[APP+'behavior_overlapTop'])
        ctl_child_mode=None
        ctl_child_mult=0.5
        if APP+'layout_collapseMode' in n.attrib:
            cm=resolve(n.attrib[APP+'layout_collapseMode']).strip()
            if cm not in {'none','pin','parallax'}:raise ValueError('Unsupported layout_collapseMode '+cm)
            ctl_child_mode=cm
        if APP+'layout_collapseParallaxMultiplier' in n.attrib:
            raw_pm=resolve(n.attrib[APP+'layout_collapseParallaxMultiplier']).strip()
            try: pm=float(raw_pm)
            except ValueError: raise ValueError('Invalid layout_collapseParallaxMultiplier '+raw_pm)
            if not 0.0<=pm<=1.0 or pm!=pm:raise ValueError('Invalid layout_collapseParallaxMultiplier '+raw_pm)
            ctl_child_mult=pm
        if parent_node in collapsing_toolbar_specs and ctl_child_mode in {'pin','parallax'}:
            relative.append('RelativeRule { kind:298, anchor:'+json.dumps(f'collapse:{ctl_child_mode}:{fmt_num(ctl_child_mult)}')+' }')
        if n in text_input_specs:
            relative.append('RelativeRule { kind:228, anchor:'+json.dumps(text_input_specs[n],ensure_ascii=False)+' }')
        if n in button_variants:
            relative.append('RelativeRule { kind:229, anchor:'+json.dumps(button_variants[n])+' }')
        if n in chip_specs:
            relative.append('RelativeRule { kind:230, anchor:'+json.dumps(chip_specs[n])+' }')
        if n in nav_bar_specs:
            relative.append('RelativeRule { kind:237, anchor:'+json.dumps(nav_bar_specs[n])+' }')
        if n in nav_item_specs:
            for nav_item in nav_item_specs[n]:
                relative.append('RelativeRule { kind:238, anchor:'+json.dumps(nav_item,ensure_ascii=False)+' }')
        if n in search_bar_specs:
            relative.append('RelativeRule { kind:239, anchor:'+json.dumps(search_bar_specs[n],ensure_ascii=False)+' }')
        if n in search_view_ext_specs:
            relative.append('RelativeRule { kind:302, anchor:'+json.dumps(search_view_ext_specs[n],ensure_ascii=False)+' }')
        if n in toggle_group_specs:
            relative.append('RelativeRule { kind:240, anchor:'+json.dumps(toggle_group_specs[n])+' }')
        if n in extended_fab_specs:
            relative.append('RelativeRule { kind:242, anchor:'+json.dumps(extended_fab_specs[n])+' }')
        if n in extended_fab_ext_specs:
            relative.append('RelativeRule { kind:299, anchor:'+json.dumps(extended_fab_ext_specs[n])+' }')
        if n in bottom_app_bar_specs:
            relative.append('RelativeRule { kind:243, anchor:'+json.dumps(bottom_app_bar_specs[n])+' }')
        if n in bottom_app_bar_ext_specs:
            relative.append('RelativeRule { kind:300, anchor:'+json.dumps(bottom_app_bar_ext_specs[n])+' }')
        if n in nav_badge_specs:
            for nav_badge in nav_badge_specs[n]:
                relative.append('RelativeRule { kind:244, anchor:'+json.dumps(nav_badge,ensure_ascii=False)+' }')
        if n in nav_badge_ext_specs:
            for nav_badge_ext in nav_badge_ext_specs[n]:
                relative.append('RelativeRule { kind:297, anchor:'+json.dumps(nav_badge_ext)+' }')
        else:
            direct_badge_spec=parse_badge_ext_attrs(n.attrib)
            if direct_badge_spec is not None:
                relative.append('RelativeRule { kind:297, anchor:'+json.dumps(f'self|{direct_badge_spec}')+' }')
        if n in collapsing_toolbar_specs:
            relative.append('RelativeRule { kind:245, anchor:'+json.dumps(collapsing_toolbar_specs[n],ensure_ascii=False)+' }')
        if n in collapsing_toolbar_ext_specs:
            relative.append('RelativeRule { kind:298, anchor:'+json.dumps(collapsing_toolbar_ext_specs[n],ensure_ascii=False)+' }')
        if n in swipe_refresh_nodes:
            relative.append('RelativeRule { kind:246, anchor:"" }')
        if n in swipe_refresh_ext_specs:
            relative.append('RelativeRule { kind:303, anchor:'+json.dumps(swipe_refresh_ext_specs[n])+' }')
        if n in drawer_layout_nodes:
            relative.append('RelativeRule { kind:247, anchor:"" }')
        if n in view_pager_specs:
            relative.append('RelativeRule { kind:248, anchor:'+json.dumps(view_pager_specs[n])+' }')
        if n in view_pager_ext_specs:
            relative.append('RelativeRule { kind:305, anchor:'+json.dumps(view_pager_ext_specs[n])+' }')
        if n in drag_handle_nodes:
            relative.append('RelativeRule { kind:260, anchor:"" }')
        if n in loading_indicator_specs:
            relative.append('RelativeRule { kind:262, anchor:'+json.dumps(loading_indicator_specs[n])+' }')
        if n in wavy_progress_specs:
            relative.append('RelativeRule { kind:263, anchor:'+json.dumps(wavy_progress_specs[n])+' }')
        if n in alert_dialog_specs:
            dlg_icon=alert_dialog_specs[n]
            if not dlg_icon:
                for desc in n.iter():
                    if desc is n or desc.tag not in {'ImageView','ImageButton'}:continue
                    if A+'id' in desc.attrib:
                        did=id_key(resolve(desc.attrib[A+'id']))
                        if did in {'android:icon','icon'}:
                            src_ref=desc.attrib.get(A+'src',desc.attrib.get(APP+'srcCompat',''))
                            dlg_icon=match_material_icon_name(src_ref)
                            if dlg_icon:break
            relative.append('RelativeRule { kind:264, anchor:'+json.dumps(dlg_icon)+' }')
        if n in snackbar_nodes:
            relative.append('RelativeRule { kind:265, anchor:"" }')
        if n in tab_options_specs:
            relative.append('RelativeRule { kind:266, anchor:'+json.dumps(tab_options_specs[n])+' }')
        if n in slider_options_specs:
            relative.append('RelativeRule { kind:267, anchor:'+json.dumps(slider_options_specs[n])+' }')
        if n in text_input_ext_specs:
            relative.append('RelativeRule { kind:268, anchor:'+json.dumps(text_input_ext_specs[n],ensure_ascii=False)+' }')
        if n in chip_group_specs:
            relative.append('RelativeRule { kind:269, anchor:'+json.dumps(chip_group_specs[n])+' }')
        if n in floating_toolbar_nodes:
            relative.append('RelativeRule { kind:270, anchor:"" }')
        if n in button_group_specs:
            relative.append('RelativeRule { kind:271, anchor:'+json.dumps(button_group_specs[n])+' }')
        if n in split_button_nodes:
            relative.append('RelativeRule { kind:272, anchor:"" }')
        if n in carousel_item_nodes:
            relative.append('RelativeRule { kind:273, anchor:"" }')
        if n in appbar_lift_specs:
            relative.append('RelativeRule { kind:279, anchor:'+json.dumps(appbar_lift_specs[n])+' }')
        if n in appbar_ext_specs:
            relative.append('RelativeRule { kind:290, anchor:'+json.dumps(appbar_ext_specs[n])+' }')
        if n in recycler_layout_ext_specs:
            relative.append('RelativeRule { kind:301, anchor:'+json.dumps(recycler_layout_ext_specs[n])+' }')
        if n in carousel_layout_specs:
            relative.append('RelativeRule { kind:304, anchor:'+json.dumps(carousel_layout_specs[n])+' }')
        for key in ('enabled','clickable','checked','indeterminate','isIndicator','measureAllChildren','fillViewport','clipToPadding','clipChildren','clipToOutline','foregroundInsidePadding','textAllCaps','mirrorForRtl','autoStart','countDown','singleLine','adjustViewBounds','textIsSelectable','fitsSystemWindows','longClickable','accessibilityHeading','animateLayoutChanges','nestedScrollingEnabled','focusableInTouchMode','includeFontPadding','linksClickable','scrollHorizontally','saveEnabled','fadeScrollbars','drawSelectorOnTop'):
            if key in a and a[key] not in {'true','false'}:raise ValueError('Unsupported boolean '+a[key])
        if 'includeFontPadding' in a and n.tag in TEXT_BEARING_TAGS:
            relative.append('RelativeRule { kind:236, anchor:'+json.dumps(a['includeFontPadding'])+' }')
        if 'outlineProvider' in a and a['outlineProvider'] not in {'none','background','bounds','paddedBounds'}:raise ValueError('Unsupported outlineProvider '+a['outlineProvider'])
        if 'fadingEdge' in a and any(p not in {'none','horizontal','vertical'} for p in a['fadingEdge'].split('|')):raise ValueError('Unsupported fadingEdge '+a['fadingEdge'])
        transform_attrs={'translationX','translationY','scaleX','scaleY','rotation','transformPivotX','transformPivotY'}
        if transform_attrs & a.keys():
            def parse_t_dp(key):
                if key not in a:return '0'
                fv=float(margin_number(a[key]))
                return str(0.0 if fv==0.0 else fv)
            def parse_t_float(key, default):
                if key not in a:return default
                try: fv=float(a[key])
                except ValueError: raise ValueError('Invalid '+key+' '+a[key])
                if fv!=fv or fv in (float('inf'),float('-inf')):raise ValueError('Invalid '+key+' '+a[key])
                return str(0.0 if fv==0.0 else fv)
            def parse_t_pivot(key):
                if key not in a:return 'center'
                raw_p=a[key].strip()
                if raw_p.endswith('%'):
                    try: pv=float(raw_p[:-1])
                    except ValueError: raise ValueError('Invalid '+key+' '+a[key])
                    if pv!=pv or pv in (float('inf'),float('-inf')):raise ValueError('Invalid '+key+' '+a[key])
                    return f'{0.0 if pv==0.0 else pv}%'
                try: fv=float(margin_number(raw_p))
                except ValueError: raise ValueError('Invalid '+key+' '+a[key])
                return str(0.0 if fv==0.0 else fv)
            t_tx=parse_t_dp('translationX')
            t_ty=parse_t_dp('translationY')
            t_sx=parse_t_float('scaleX','1')
            t_sy=parse_t_float('scaleY','1')
            t_rot=parse_t_float('rotation','0')
            t_px=parse_t_pivot('transformPivotX')
            t_py=parse_t_pivot('transformPivotY')
            relative.append('RelativeRule { kind:256, anchor:'+json.dumps(f'{t_tx},{t_ty},{t_sx},{t_sy},{t_rot},{t_px},{t_py}')+' }')
        if 'tooltipText' in a and a['tooltipText'] and a['tooltipText']!='@null':
            relative.append('RelativeRule { kind:258, anchor:'+json.dumps(a['tooltipText'],ensure_ascii=False)+' }')
        if 'overScrollMode' in a and a['overScrollMode'] not in {'always','ifContentScrolls','never'}:raise ValueError('Unsupported overScrollMode '+a['overScrollMode'])
        auto_link_mask='none'
        if 'autoLink' in a:
            parts=[p.strip() for p in a['autoLink'].split('|')]
            if not parts or any(p not in {'none','web','email','phone','map','all'} for p in parts):raise ValueError('Unsupported autoLink '+a['autoLink'])
            pset=set(parts)
            if 'all' in pset:
                auto_link_mask='all'
            else:
                ordered=[f for f in ('web','email','phone','map') if f in pset]
                auto_link_mask=','.join(ordered) if ordered else 'none'
        link_color_rgba=''
        if 'textColorLink' in a and a['textColorLink']!='@null':
            link_color_rgba=rgba_to_csv(static_color_rgba(a['textColorLink']))
        if n.tag in TEXT_BEARING_TAGS and (auto_link_mask!='none' or link_color_rgba):
            links_clickable='0' if a.get('linksClickable','true')=='false' else '1'
            relative.append('RelativeRule { kind:306, anchor:'+json.dumps(f'{auto_link_mask}|{link_color_rgba}|{links_clickable}')+' }')
        if 'typeface' in a and a['typeface'] not in {'normal','sans','serif','monospace'}:raise ValueError('Unsupported typeface '+a['typeface'])
        if 'lineHeight' in a:
            lh_raw=resolve(a['lineHeight']).strip()
            lh_tok=theme_token(lh_raw)
            if lh_tok is not None:
                if lh_tok not in THEME_DIMENS:raise ValueError('Unresolved theme dimension '+lh_raw)
                lh_raw=THEME_DIMENS[lh_tok]
            if not re.fullmatch(r'(?:\d+(?:\.\d+)?|\.\d+)(?:dp|dip|sp|px)',lh_raw):raise ValueError('Invalid lineHeight '+a['lineHeight'])
            lh_val=float(re.sub(r'(?:dp|dip|sp|px)$','',lh_raw))
            if lh_val>0.0 and n.tag in TEXT_BEARING_TAGS:
                relative.append('RelativeRule { kind:235, anchor:'+json.dumps(str(lh_val))+' }')
        break_strat='simple'
        if 'breakStrategy' in a:
            bs_val=a['breakStrategy'].strip()
            if bs_val not in {'simple','high_quality','balanced'}:raise ValueError('Unsupported breakStrategy '+a['breakStrategy'])
            break_strat=bs_val
        hyphen_freq='none'
        if 'hyphenationFrequency' in a:
            hf_val=a['hyphenationFrequency'].strip()
            hf_norm={'none':'none','normal':'normal','full':'full','normalFast':'normal','fullFast':'full'}.get(hf_val)
            if hf_norm is None:raise ValueError('Unsupported hyphenationFrequency '+a['hyphenationFrequency'])
            hyphen_freq=hf_norm
        just_mode='none'
        if 'justificationMode' in a:
            jm_val=a['justificationMode'].strip()
            if jm_val not in {'none','inter_word'}:raise ValueError('Unsupported justificationMode '+a['justificationMode'])
            just_mode=jm_val
        auto_type='none'
        if 'autoSizeTextType' in a:
            at_val=a['autoSizeTextType'].strip()
            if at_val not in {'none','uniform'}:raise ValueError('Unsupported autoSizeTextType '+a['autoSizeTextType'])
            auto_type=at_val
        def parse_autosize_dim(attr_name, raw_val):
            s=resolve(raw_val).strip()
            tok=theme_token(s)
            if tok is not None:
                if tok not in THEME_DIMENS:raise ValueError('Unresolved theme dimension '+s)
                s=THEME_DIMENS[tok]
            if not re.fullmatch(r'(?:\d+(?:\.\d+)?|\.\d+)(?:dp|dip|sp|px)',s):
                raise ValueError('Invalid '+attr_name+' '+raw_val)
            val=float(re.sub(r'(?:dp|dip|sp|px)$','',s))
            if not (val>0.0) or val!=val or val==float('inf'):
                raise ValueError('Invalid '+attr_name+' '+raw_val)
            return val
        auto_min_val=parse_autosize_dim('autoSizeMinTextSize',a['autoSizeMinTextSize']) if 'autoSizeMinTextSize' in a else None
        auto_max_val=parse_autosize_dim('autoSizeMaxTextSize',a['autoSizeMaxTextSize']) if 'autoSizeMaxTextSize' in a else None
        auto_step_val=parse_autosize_dim('autoSizeStepGranularity',a['autoSizeStepGranularity']) if 'autoSizeStepGranularity' in a else None
        if auto_type=='uniform':
            auto_min=auto_min_val if auto_min_val is not None else 12.0
            auto_max=auto_max_val if auto_max_val is not None else 112.0
            auto_step=auto_step_val if auto_step_val is not None else 1.0
            if auto_min>auto_max:
                raise ValueError('Invalid autoSizeMinTextSize exceeds autoSizeMaxTextSize')
            auto_spec=f'{fmt_num(auto_min)},{fmt_num(auto_max)},{fmt_num(auto_step)}'
        else:
            if auto_min_val is not None and auto_max_val is not None and auto_min_val>auto_max_val:
                raise ValueError('Invalid autoSizeMinTextSize exceeds autoSizeMaxTextSize')
            auto_spec=''
        if n.tag in TEXT_BEARING_TAGS and (break_strat!='simple' or hyphen_freq!='none' or just_mode!='none' or auto_spec):
            relative.append('RelativeRule { kind:296, anchor:'+json.dumps(f'{break_strat}|{hyphen_freq}|{just_mode}|{auto_spec}')+' }')
        if 'imeOptions' in a:
            valid_ime={'normal','actionUnspecified','actionNone','actionGo','actionSearch','actionSend','actionNext','actionDone','actionPrevious','flagNoPersonalizedLearning','flagNoFullscreen','flagNavigatePrevious','flagNavigateNext','flagNoExtractUi','flagNoAccessoryAction','flagNoEnterAction','flagForceAscii'}
            if any(p not in valid_ime for p in a['imeOptions'].split('|')):raise ValueError('Unsupported imeOptions '+a['imeOptions'])
        if 'descendantFocusability' in a and a['descendantFocusability'] not in {'beforeDescendants','afterDescendants','blocksDescendants'}:raise ValueError('Unsupported descendantFocusability '+a['descendantFocusability'])
        if n.tag not in {'ViewFlipper','AdapterViewFlipper'} and ('autoStart' in a or 'flipInterval' in a):raise ValueError('ViewFlipper attributes unsupported on '+n.tag)
        interval=a.get('flipInterval','10000' if n.tag=='AdapterViewFlipper' else '3000')
        if not re.fullmatch(r'\d+',interval) or not 1<=int(interval)<=3600000:raise ValueError('Invalid flipInterval '+interval)
        if 'mirrorForRtl' in a and n.tag not in {'RatingBar','SeekBar','ProgressBar'}:raise ValueError('mirrorForRtl unsupported on '+n.tag)
        def progress_integer(name, default):
            value=a.get(name,default)
            if not re.fullmatch(r'-?\d+',value):raise ValueError('Expected integer '+name+': '+value)
            number=int(value)
            if not -16777216 <= number <= 16777216:raise ValueError('Integer outside supported range: '+name)
            return number
        minimum=progress_integer('min','0');maximum=progress_integer('max','100');progress=progress_integer('progress','0');secondary=progress_integer('secondaryProgress','0')
        if minimum > maximum:raise ValueError('SeekBar minimum exceeds maximum')
        if 'tickMark' in a and maximum-minimum>1024:raise ValueError('tickMark range exceeds 1024')
        progress=max(minimum,min(maximum,progress));secondary=max(minimum,min(maximum,secondary))
        if n in range_slider_specs:
            rs_vals=range_slider_specs[n]
            if rs_vals is None:
                v0=minimum+0.25*(maximum-minimum)
                v1=minimum+0.75*(maximum-minimum)
            else:
                v0=rs_vals[0]
                v1=rs_vals[1] if len(rs_vals)>=2 else float(maximum)
                v0=max(float(minimum),min(float(maximum),v0))
                v1=max(float(minimum),min(float(maximum),v1))
                if v0>v1:v0,v1=v1,v0
            if v0==0.0:v0=0.0
            if v1==0.0:v1=0.0
            relative.append('RelativeRule { kind:250, anchor:'+json.dumps(f'{v0},{v1}')+' }')
        id=(id_key(a['id']) if a['id'].startswith('@') else a['id']) if 'id' in a else ''
        if id is None:raise ValueError('Invalid view ID '+a['id'])
        if id and id in ids:raise ValueError('Duplicate id '+id)
        ids.add(id) if id else None
        try:
            stars=int(a.get('numStars','5')); rating=float(a.get('rating','0')); step=float(a.get('stepSize','0.5'))
        except ValueError:raise ValueError('Invalid RatingBar values')
        if not 1 <= stars <= 20 or not 0 <= rating <= stars or not 0 < step <= stars:raise ValueError('RatingBar values outside supported range')
        checked_button=a.get('checkedButton','')
        if checked_button:
            if n.tag!='RadioGroup' or not re.fullmatch(r'@(?:\+)?id/[A-Za-z_][A-Za-z0-9_]*',checked_button):raise ValueError('Invalid checkedButton reference')
            checked_button=checked_button.split('/')[-1]
            matches=[child for child in n.iter() if child is not n and child.attrib.get(A+'id','').split('/')[-1]==checked_button]
            if len(matches)!=1 or matches[0].tag!='RadioButton':raise ValueError('checkedButton must refer to a uniquely identified RadioButton descendant')
        visibility=a.get('visibility','visible')
        if visibility not in {'visible','invisible','gone'}:raise ValueError('Unsupported visibility '+visibility)
        if n.tag!='Chronometer' and ('format' in a or 'countDown' in a):raise ValueError('Chronometer attributes unsupported on '+n.tag)
        if n.tag!='TextClock' and ('format12Hour' in a or 'format24Hour' in a):raise ValueError('TextClock attributes unsupported on '+n.tag)
        for pattern in (a.get('format12Hour',''),a.get('format24Hour','')):
            if pattern.count("'")%2:raise ValueError('Unclosed TextClock literal')
            unquoted=re.sub(r"'[^']*'",'',pattern)
            if re.search(r'[A-Za-z]',unquoted.translate(str.maketrans('', '', 'HhKkmsayMd'))):raise ValueError('Unsupported TextClock pattern '+pattern)
            if re.search(r'M{3,}|d{3,}|H{3,}|h{3,}|K{3,}|k{3,}|m{3,}|s{3,}|y{5,}',unquoted):raise ValueError('Unsupported TextClock pattern '+pattern)
        chrono_format=a.get('format','')
        if chrono_format:
            tokens=re.findall(r'%(?:1\$s|s|%|n)',chrono_format)
            rest=re.sub(r'%(?:1\$s|s|%|n)','',chrono_format)
            if '%' in rest or tokens.count('%s')>1:raise ValueError('Unsupported Chronometer format '+chrono_format)
        if ('maxWidth' in a or 'maxHeight' in a) and n.tag not in {'ImageView','ImageButton','ZoomButton','QuickContactBadge'}:raise ValueError('image maximum dimensions unsupported on '+n.tag)
        tint_states=tint_selector(a['tint']) if 'tint' in a else None
        text_color_states=tint_selector(a['textColor']) if 'textColor' in a else None
        if text_color_states and n.tag not in {'TextView','CheckedTextView','Chronometer','TextClock','DigitalClock','Button','CheckBox','RadioButton','Switch','ToggleButton'}:raise ValueError('text color selector unsupported on '+n.tag)
        if ('tint' in a or 'tintMode' in a) and n.tag not in {'ImageView','ImageButton','ZoomButton','QuickContactBadge'}:raise ValueError('image tint unsupported on '+n.tag)
        if a.get('tintMode','src_atop') not in {'src_atop','src_in','src_over','multiply','screen','add'}:raise ValueError('Unsupported image tintMode '+a['tintMode'])
        if 'adjustViewBounds' in a and n.tag not in {'ImageView','ImageButton','ZoomButton','QuickContactBadge'}:raise ValueError('adjustViewBounds unsupported on '+n.tag)
        if 'imageAlpha' in a and n.tag not in {'ImageView','ImageButton','ZoomButton','QuickContactBadge'}:raise ValueError('imageAlpha unsupported on '+n.tag)
        image_alpha=a.get('imageAlpha','255')
        if not image_alpha.isdigit() or not 0<=int(image_alpha)<=255:raise ValueError('Invalid imageAlpha '+image_alpha)
        view_alpha=resolve(a.get('alpha','1'))
        if not re.fullmatch(r'(?:0(?:\.\d+)?|1(?:\.0+)?)',view_alpha):raise ValueError('Invalid alpha '+view_alpha)
        letter_spacing=resolve(a.get('letterSpacing','0'))
        if not re.fullmatch(r'-?(?:\d+(?:\.\d+)?|\.\d+)',letter_spacing) or abs(float(letter_spacing))>100:raise ValueError('Invalid letterSpacing '+letter_spacing)
        spacing_multiplier=resolve(a.get('lineSpacingMultiplier','1'))
        if not re.fullmatch(r'(?:\d+(?:\.\d+)?|\.\d+)',spacing_multiplier) or not 0<float(spacing_multiplier)<=100:raise ValueError('Invalid lineSpacingMultiplier '+spacing_multiplier)
        raw_input_type=a.get('inputType','text')
        input_parts=[p.strip() for p in raw_input_type.split('|')]
        if not input_parts or any(p not in VALID_INPUT_TYPE_FLAGS for p in input_parts):raise ValueError('Unsupported inputType '+raw_input_type)
        input_pset=set(input_parts)
        if input_pset & {'textPassword','textWebPassword','numberPassword'}:input_type_code=4
        elif input_pset & {'textEmailAddress','textWebEmailAddress'}:input_type_code=1
        elif 'phone' in input_pset:input_type_code=2
        elif 'numberDecimal' in input_pset:input_type_code=5
        elif input_pset & {'number','numberSigned'}:input_type_code=3
        else:input_type_code=0
        if 'inputType' in a and n.tag not in {'EditText','AutoCompleteTextView','MultiAutoCompleteTextView'}:raise ValueError('inputType unsupported on '+n.tag)
        if n.tag not in {'ViewAnimator','ViewSwitcher','TextSwitcher','ImageSwitcher','ViewFlipper','AdapterViewFlipper'} and ('inAnimation' in a or 'outAnimation' in a):raise ValueError('Animations unsupported on '+n.tag)
        anim_in,translate_in,scale_in,rotate_in,elev_in=(object_alpha_animator if n.tag=='AdapterViewFlipper' else alpha_animation)(a['inAnimation']) if 'inAnimation' in a else ('None','None',None,None,None)
        anim_out,translate_out,scale_out,rotate_out,elev_out=(object_alpha_animator if n.tag=='AdapterViewFlipper' else alpha_animation)(a['outAnimation']) if 'outAnimation' in a else ('None','None',None,None,None)
        if scale_in is not None:relative.append('RelativeRule { kind:274, anchor:'+json.dumps(scale_in)+' }')
        if scale_out is not None:relative.append('RelativeRule { kind:275, anchor:'+json.dumps(scale_out)+' }')
        if rotate_in is not None:relative.append('RelativeRule { kind:276, anchor:'+json.dumps(rotate_in)+' }')
        if rotate_out is not None:relative.append('RelativeRule { kind:277, anchor:'+json.dumps(rotate_out)+' }')
        if elev_in is not None:relative.append('RelativeRule { kind:288, anchor:'+json.dumps(elev_in)+' }')
        if elev_out is not None:relative.append('RelativeRule { kind:289, anchor:'+json.dumps(elev_out)+' }')
        text_style=a.get('textStyle','normal')
        style_is_bold=parse_text_style(text_style)
        style_is_italic='italic' in [p.strip() for p in text_style.split('|')]
        if n.tag not in TEXT_BEARING_TAGS and ('textStyle' in a or 'textAllCaps' in a):raise ValueError('Text styling unsupported on '+n.tag)
        font_family=a.get('fontFamily','').strip()
        text_is_bold=style_is_bold or (font_family in {'sans-serif-medium','sans-serif-black'} and n.tag in TEXT_BEARING_TAGS)
        if n.tag in TEXT_BEARING_TAGS:
            tf=a.get('typeface','').strip()
            if font_family:
                low_ff=font_family.lower()
                if low_ff=='monospace' or 'monospace' in low_ff or low_ff=='mono':text_family='mono'
                elif low_ff=='serif' or low_ff.startswith('serif-'):text_family='serif'
                else:text_family='sans'
            elif tf=='monospace':text_family='mono'
            elif tf=='serif':text_family='serif'
            else:text_family='sans'
            if style_is_italic or text_family!='sans':
                relative.append('RelativeRule { kind:257, anchor:'+json.dumps(f'{"1" if style_is_italic else "0"}|{text_family}')+' }')
        if 'textAllCaps' in a:
            relative.append('RelativeRule { kind:218, anchor:'+json.dumps(a['textAllCaps'])+' }')
        min_lines=a.get('minLines','1')
        if not re.fullmatch(r'\d+',min_lines) or not 1<=int(min_lines)<=10000:raise ValueError('Invalid minLines '+min_lines)
        max_lines=a.get('maxLines','0')
        if not re.fullmatch(r'\d+',max_lines) or int(max_lines)>10000 or ('maxLines' in a and int(max_lines)==0):raise ValueError('Invalid maxLines '+max_lines)
        if 'lines' in a:
            if n.tag not in {'TextView','CheckedTextView','Chronometer','TextClock','DigitalClock','EditText','AutoCompleteTextView','MultiAutoCompleteTextView'}:raise ValueError('lines unsupported on '+n.tag)
            lines=a['lines']
            if not re.fullmatch(r'\d+',lines) or not 1<=int(lines)<=10000:raise ValueError('Invalid lines '+lines)
            # Framework TextView applies maxLines, then lines, then minLines,
            # in resource-ID order, regardless of XML attribute order.
            max_lines=lines
            if 'minLines' not in a:min_lines=lines
        ellipsize=a.get('ellipsize','none')
        if ellipsize not in {'none','end','start','middle','marquee'}:raise ValueError('Unsupported ellipsize '+ellipsize)
        if 'singleLine' in a and n.tag not in {'TextView','CheckedTextView','Chronometer','TextClock'}:raise ValueError('singleLine unsupported on '+n.tag)
        single_line=a.get('singleLine','false')=='true' or ellipsize=='marquee'
        if single_line:max_lines='1';min_lines='1'
        if ellipsize in {'start','middle'} and max_lines!='1':raise ValueError('start/middle ellipsize requires one line')
        direction=a.get('layoutDirection','inherit')
        if direction not in {'inherit','ltr','rtl'}:raise ValueError('Unsupported layoutDirection '+direction)
        orientation=a.get('orientation','horizontal' if n.tag=='GridLayout' else 'vertical')
        if orientation not in {'vertical','horizontal'}:raise ValueError('Unsupported orientation')
        if 'measureAllChildren' in a and n.tag not in {'FrameLayout','MediaController','ViewAnimator','ViewSwitcher','TextSwitcher','ImageSwitcher','ViewFlipper'}:raise ValueError('measureAllChildren unsupported on '+n.tag)
        if n.tag not in {'LinearLayout','RadioGroup','TabWidget'} and 'weightSum' in a:raise ValueError('weightSum unsupported on '+n.tag)
        if n.tag not in {'LinearLayout','RadioGroup','ListView','ExpandableListView'} and any(k in a for k in ('divider','dividerPadding','showDividers')):raise ValueError('Divider unsupported on '+n.tag)
        divider_color,divider_thickness=divider_drawable(a['divider'],orientation=='horizontal') if 'divider' in a else ('None','0.0')
        if n.tag in {'ListView','ExpandableListView'}:
            if 'dividerPadding' in a or 'showDividers' in a:raise ValueError('Unsupported ListView divider attribute')
            if 'dividerHeight' in a:divider_thickness=number(a['dividerHeight'])
        if material_divider:
            divider_thickness=number(n.attrib.get(APP+'dividerThickness','1dp'))
            divider_color=color(n.attrib[APP+'dividerColor']) if APP+'dividerColor' in n.attrib and n.attrib[APP+'dividerColor']!='?attr/colorOutlineVariant' else 'None'
        flags=a.get('showDividers','none').split('|')
        if len(set(flags))!=len(flags) or any(v not in {'none','beginning','middle','end'} for v in flags) or ('none' in flags and len(flags)>1):raise ValueError('Invalid showDividers '+a.get('showDividers',''))
        divider_flags=sum({'none':0,'beginning':1,'middle':2,'end':4}[v] for v in flags)

        def parse_gravity(raw, name):
            h=None;v=None;has_center=False
            h_axes={'start':0,'left':3,'end':2,'right':4,'center_horizontal':1}
            v_axes={'top':0,'bottom':2,'center_vertical':1}
            for part in raw.split('|'):
                part=part.strip()
                if part=='center':
                    has_center=True
                elif part in h_axes:
                    value=h_axes[part]
                    if h is not None and h!=value:
                        if {h,value}=={0,3}:h=0
                        elif {h,value}=={2,4}:h=2
                        else:raise ValueError('Conflicting horizontal '+name)
                    else:h=value
                elif part in v_axes:
                    value=v_axes[part]
                    if v is not None and v!=value:raise ValueError('Conflicting vertical '+name)
                    v=value
                else:
                    raise ValueError('Unsupported '+name+' '+part)
            if has_center:
                if h is None:h=1
                if v is None:v=1
            return (h if h is not None else 0, v if v is not None else 0)
        def foreground_gravity(raw):
            if raw=='fill':return (5,5)
            h=None;v=None;has_center=False
            h_axes={'start':0,'left':3,'end':2,'right':4,'center_horizontal':1,'fill_horizontal':5}
            v_axes={'top':0,'bottom':2,'center_vertical':1,'fill_vertical':5}
            for part in raw.split('|'):
                part=part.strip()
                if part=='center':
                    has_center=True
                elif part in h_axes:
                    value=h_axes[part]
                    if h is not None and h!=value:
                        if {h,value}=={0,3}:h=0
                        elif {h,value}=={2,4}:h=2
                        else:raise ValueError('Conflicting foregroundGravity')
                    else:h=value
                elif part in v_axes:
                    value=v_axes[part]
                    if v is not None and v!=value:raise ValueError('Conflicting foregroundGravity')
                    v=value
                else:
                    raise ValueError('Unsupported foregroundGravity '+part)
            if has_center:
                if h is None:h=1
                if v is None:v=1
            return (h if h is not None else 0,v if v is not None else 0)
        fg_h,fg_v=foreground_gravity(a['foregroundGravity']) if 'foregroundGravity' in a else (5,5)
        if parent=='GridLayout' and 'layout_gravity' in a:
            parts=a['layout_gravity'].split('|');baseline='baseline' in parts
            horizontal_gravity,vertical_gravity=foreground_gravity('|'.join('top' if part=='baseline' else part for part in parts))
            if baseline:vertical_gravity=6
        else:horizontal_gravity,vertical_gravity=parse_gravity(a.get('layout_gravity','start|top'),'layout_gravity')
        content_h,content_v=parse_gravity(a.get('gravity','start|top'),'gravity')
        src=a.get('navigationIcon','') if n.tag=='Toolbar' else a.get('src','')
        if src=='@null':src=''
        if n.tag in {'ImageView','ImageButton','ZoomButton','Toolbar','QuickContactBadge'} and src and not re.fullmatch(r'@drawable/[a-z][a-z0-9_]*',src):raise ValueError('Unsupported image src '+src)
        scale_type=a.get('scaleType','fitCenter')
        if n.tag in {'ImageView','ImageButton','ZoomButton','QuickContactBadge'} and scale_type not in {'fitCenter','centerCrop','fitXY','center','centerInside','fitStart','fitEnd','matrix'}:raise ValueError('Unsupported ImageView scaleType '+scale_type)
        icon=src.split('/')[-1] if src else 'menu'
        icons={'menu':'Menu','search':'Search','more_vert':'MoreVert','add':'Add','arrow_back':'ArrowBack','send':'ArrowForward','settings':'Settings','close':'Close'}
        vec_default_tint='None'
        vec_default_tint_mode=1
        src_auto_mirrored=False
        src_stateful_images=[]
        src_animation_list=None
        src_dynamic_drawable=None
        src_animated_transitions=[]
        src_animated_vector=None
        def resolve_image_element(xroot, dref, target_name, seen, xml_file=None, in_selector=False):
            nonlocal vec_default_tint, vec_default_tint_mode, src_animation_list, src_dynamic_drawable, src_animated_vector
            if xroot.tag=='vector':
                _,_,vtint,_,v_am=resolve_and_rasterize_vector(xml_file if xml_file is not None else xroot,target_name)
                if not in_selector and vtint is not None and vec_default_tint=='None' and n.tag in {'ImageView','ImageButton','ZoomButton','QuickContactBadge'}:
                    vec_default_tint=rgba_to_rust(vtint)
                    vec_default_tint_mode={'src_atop':0,'src_in':1,'src_over':2,'multiply':3,'screen':4,'add':5}.get(vector_tint_modes.get(target_name,'src_in'),1)
                return (True,'',v_am)
            if xroot.tag=='bitmap':
                ba=strip_tools(xroot.attrib)
                allowed_bmp={A+'src',A+'tint',A+'tintMode',A+'gravity',A+'tileMode',A+'tileModeX',A+'tileModeY',A+'antialias',A+'dither',A+'filter',A+'mipMap',A+'alpha',A+'autoMirrored',A+'visible'}
                if len(xroot) or (set(ba)-allowed_bmp) or A+'src' not in ba:raise ValueError('Invalid bitmap drawable: '+dref)
                if A+'gravity' in ba:
                    gparts=[p.strip() for p in resolve(ba[A+'gravity']).split('|') if p.strip()]
                    if not gparts or any(p not in BITMAP_GRAVITY_TOKENS for p in gparts):raise ValueError('Unsupported bitmap gravity '+ba[A+'gravity'])
                if A+'tintMode' in ba and resolve(ba[A+'tintMode']).strip() not in {'src_atop','src_in','src_over','multiply','screen','add'}:
                    raise ValueError('Unsupported bitmap tintMode '+ba[A+'tintMode'])
                ok,st,am=resolve_image_drawable(resolve(ba[A+'src']),target_name,seen+(dref,),in_selector=in_selector)
                if ok and A+'tint' in ba:
                    b_rgba=static_color_rgba(ba[A+'tint'])
                    r,g,b,al=b_rgba
                    indicator_tints[target_name]=f'#{al:02x}{r:02x}{g:02x}{b:02x}'
                    if not in_selector and vec_default_tint=='None' and n.tag in {'ImageView','ImageButton','ZoomButton','QuickContactBadge'}:
                        vec_default_tint=rgba_to_rust(b_rgba)
                        if A+'tintMode' in ba:
                            vec_default_tint_mode={'src_atop':0,'src_in':1,'src_over':2,'multiply':3,'screen':4,'add':5}.get(resolve(ba[A+'tintMode']).strip(),1)
                return (ok,st,am)
            if xroot.tag in {'rotate','scale','clip'}:
                kind,payload,wrap_am=validate_drawable_wrapper(xroot,dref)
                if not in_selector and src_dynamic_drawable is None:
                    src_dynamic_drawable=decode_dynamic_drawable(xroot,dref,target_name,seen)
                ok,st,am=(resolve_image_drawable(payload,target_name,seen+(dref,),in_selector=in_selector) if kind=='ref'
                    else resolve_image_element(payload,dref,target_name,seen,in_selector=in_selector))
                return (ok,st,am or wrap_am)
            if xroot.tag=='animation-list':
                anim_spec,first_key,first_st,any_am=decode_animation_list_frames(xroot,dref,target_name,seen)
                if first_key in images and target_name not in images:
                    images[target_name]=images[first_key]
                if first_key in indicator_tints and target_name not in indicator_tints:
                    indicator_tints[target_name]=indicator_tints[first_key]
                if first_key in vector_tint_modes and target_name not in vector_tint_modes:
                    vector_tint_modes[target_name]=vector_tint_modes[first_key]
                if not in_selector and first_key in indicator_tints and vec_default_tint=='None' and n.tag in {'ImageView','ImageButton','ZoomButton','QuickContactBadge'}:
                    ft=indicator_tints[first_key]
                    vt_rgba=static_color_rgba('?attr/colorControlNormal') if ft=='controlNormal' else parse_color_rgba(ft)
                    vec_default_tint=rgba_to_rust(vt_rgba)
                    vec_default_tint_mode={'src_atop':0,'src_in':1,'src_over':2,'multiply':3,'screen':4,'add':5}.get(vector_tint_modes.get(first_key,'src_in'),1)
                if not in_selector:
                    src_animation_list=anim_spec
                return (True,first_st,any_am)
            if xroot.tag in {'level-list','transition'}:
                kind,payload,list_am=select_list_drawable_item(xroot,dref)
                if not in_selector and src_dynamic_drawable is None:
                    src_dynamic_drawable=decode_dynamic_drawable(xroot,dref,target_name,seen)
                ok,st,am=(resolve_image_drawable(payload,target_name,seen+(dref,),in_selector=in_selector) if kind=='ref'
                    else resolve_image_element(payload,dref,target_name,seen,in_selector=in_selector))
                return (ok,st,am or list_am)
            if xroot.tag=='inset':
                ia=strip_tools(xroot.attrib)
                if A+'drawable' in ia and not len(xroot):
                    return resolve_image_drawable(resolve(ia[A+'drawable']),target_name,seen+(dref,),in_selector=in_selector)
                if A+'drawable' not in ia and len(xroot)==1:
                    return resolve_image_element(xroot[0],dref,target_name,seen,in_selector=in_selector)
                raise ValueError('Invalid inset drawable: '+dref)
            if xroot.tag=='layer-list':
                items=list(xroot)
                if not items or any(it.tag!='item' for it in items):raise ValueError('Unsupported layer-list child: '+dref)
                for it in items:
                    ia=strip_tools(it.attrib)
                    if not ((A+'drawable' in ia and not len(it)) or (A+'drawable' not in ia and len(it)==1)):
                        raise ValueError('Invalid layer-list item: '+dref)
                for it in reversed(items):
                    ia=strip_tools(it.attrib)
                    if A+'drawable' in ia:
                        dr=resolve(ia[A+'drawable'])
                        if dr=='@null':continue
                        ok,st,am=resolve_image_drawable(dr,target_name,seen+(dref,),in_selector=in_selector)
                    else:
                        ok,st,am=resolve_image_element(it[0],dref,target_name,seen,in_selector=in_selector)
                    if ok:return (ok,st,am)
                return (False,'',False)
            if xroot.tag=='ripple':
                ra=strip_tools(xroot.attrib)
                if A+'color' not in ra:raise ValueError('Ripple drawable requires android:color: '+dref)
                static_color_rgba(ra[A+'color'])
                content_items=[]
                for child in xroot:
                    if child.tag!='item':raise ValueError('Unsupported ripple child: '+dref)
                    if child.attrib.get(A+'id')!='@android:id/mask':content_items.append(child)
                for it in reversed(content_items):
                    ia=strip_tools(it.attrib)
                    if A+'drawable' in ia and not len(it):
                        dr=resolve(ia[A+'drawable'])
                        if dr=='@null':continue
                        ok,st,am=resolve_image_drawable(dr,target_name,seen+(dref,),in_selector=in_selector)
                    elif A+'drawable' not in ia and len(it)==1:
                        ok,st,am=resolve_image_element(it[0],dref,target_name,seen,in_selector=in_selector)
                    else:
                        raise ValueError('Invalid ripple content item: '+dref)
                    if ok:return (ok,st,am)
                return (False,'',False)
            if xroot.tag=='color':
                ca=strip_tools(xroot.attrib)
                if len(xroot) or set(ca)!={A+'color'}:raise ValueError('Unsupported color drawable: '+dref)
                r,g,b,al=static_color_rgba(ca[A+'color'])
                if target_name not in images:
                    px_sz=max(1,int(round(24.0*density/160.0)))
                    images[target_name]=(px_sz,px_sz,bytes([r,g,b,al])*(px_sz*px_sz),160.0/density)
                return (True,'',False)
            if xroot.tag=='animated-vector':
                avd_spec,first_key,_,_,vtint,v_am=decode_animated_vector_frames(xroot,dref,target_name,seen,sample_frames=(not in_selector))
                if first_key in images and target_name not in images:
                    images[target_name]=images[first_key]
                if first_key in indicator_tints and target_name not in indicator_tints:
                    indicator_tints[target_name]=indicator_tints[first_key]
                if first_key in vector_tint_modes and target_name not in vector_tint_modes:
                    vector_tint_modes[target_name]=vector_tint_modes[first_key]
                if not in_selector and vtint is not None and vec_default_tint=='None' and n.tag in {'ImageView','ImageButton','ZoomButton','QuickContactBadge'}:
                    vec_default_tint=rgba_to_rust(vtint)
                    vec_default_tint_mode={'src_atop':0,'src_in':1,'src_over':2,'multiply':3,'screen':4,'add':5}.get(vector_tint_modes.get(first_key,'src_in'),1)
                if not in_selector and avd_spec is not None:
                    src_animated_vector=avd_spec
                return (True,'',v_am)
            if xroot.tag in {'selector','animated-selector'}:
                sel_flags={'state_enabled':1,'state_pressed':2,'state_focused':4,'state_selected':8,'state_activated':16,'state_hovered':32,'state_checked':64}
                entries=[];any_am=False
                item_id_to_mask={}
                for idx,item in enumerate(xroot):
                    if xroot.tag=='animated-selector' and item.tag=='transition':continue
                    if item.tag!='item':raise ValueError('Invalid selector item: '+dref)
                    ia=strip_tools(item.attrib)
                    req=forbid=0;unknown_state=False
                    for attr,raw_val in ia.items():
                        if attr in {A+'drawable',A+'color',A+'alpha',A+'id'}:continue
                        sname=attr[len(A):] if attr.startswith(A) else ''
                        if sname in sel_flags:
                            bv=resolve(raw_val)
                            if bv not in {'true','false'}:raise ValueError('Invalid selector boolean '+bv)
                            if bv=='true':req|=sel_flags[sname]
                            else:forbid|=sel_flags[sname]
                        else:
                            unknown_state=True
                    if xroot.tag=='animated-selector' and A+'id' in ia:
                        raw_iid=resolve(ia[A+'id']).strip()
                        iid=id_key(raw_iid)
                        if iid is None:raise ValueError('Invalid animated-selector item id '+raw_iid)
                        if not unknown_state:item_id_to_mask[iid]=req
                    item_key='';item_st='';item_ok=False;item_am=False
                    if A+'drawable' in ia and not len(item):
                        dr=resolve(ia[A+'drawable'])
                        if dr=='@null':
                            item_ok=True
                        elif dr.startswith('@drawable/'):
                            item_key=dr.split('/')[-1]
                            item_ok,item_st,item_am=resolve_image_drawable(dr,item_key,seen+(dref,),in_selector=True)
                            if not item_ok:raise ValueError('Unresolved PNG drawable '+dr)
                    elif A+'drawable' not in ia and len(item)==1:
                        item_key=f'{target_name}__s{idx}'
                        item_ok,item_st,item_am=resolve_image_element(item[0],dref,item_key,seen,in_selector=True)
                    if item_ok and not unknown_state:
                        entries.append((req,forbid,item_key,item_st))
                        any_am=any_am or item_am
                if xroot.tag=='animated-selector' and not in_selector:
                    trs,tr_am=decode_animated_selector_transitions(xroot,dref,target_name,item_id_to_mask,seen)
                    src_animated_transitions.extend(trs)
                    any_am=any_am or tr_am
                if entries:
                    if not in_selector:
                        src_stateful_images.extend(entries)
                    default_entry=next((e for e in entries if e[0]==0 and e[1]==0 and e[2]),None)
                    if default_entry is None:
                        default_entry=next((e for e in entries if e[0]==1 and e[1]==0 and e[2]),None)
                    if default_entry is None:
                        default_entry=next((e for e in entries if e[2]),None)
                    if default_entry is not None and default_entry[2] in images:
                        if target_name not in images:images[target_name]=images[default_entry[2]]
                        if default_entry[2] in indicator_tints and target_name not in indicator_tints:
                            indicator_tints[target_name]=indicator_tints[default_entry[2]]
                    return (True,default_entry[3] if default_entry is not None else '',any_am)
                return (False,'',False)
            if xroot.tag=='shape':
                sd=parse_shape_element(xroot,dref)
                sw_dp=sd['size'][0] if sd['size'][0]>0 else 24.0
                sh_dp=sd['size'][1] if sd['size'][1]>0 else 24.0
                vec=ET.Element('vector',{A+'width':f'{sw_dp}dp',A+'height':f'{sh_dp}dp',A+'viewportWidth':str(sw_dp),A+'viewportHeight':str(sh_dp)})
                stw=float(sd['stroke_width']) if sd['stroke_rgba'] is not None else 0.0
                ins=stw*0.5;iw=max(0.1,sw_dp-stw);ih=max(0.1,sh_dp-stw)
                if sd['shape_kind']=='line':
                    cy=sh_dp*0.5
                    pdata=f'M0,{cy} L{sw_dp},{cy}'
                elif sd['radius']=='-1.0':
                    rx=iw*0.5;ry=ih*0.5;cx=sw_dp*0.5;cy=sh_dp*0.5
                    pdata=f'M{cx},{cy-ry} A{rx},{ry} 0 1,0 {cx},{cy+ry} A{rx},{ry} 0 1,0 {cx},{cy-ry} Z'
                else:
                    if sd['corner_radii'] is not None:
                        tl,tr,br,bl=[min(float(x),iw*0.5,ih*0.5) for x in sd['corner_radii'].split(',')]
                    else:
                        rv=min(max(0.0,float(sd['radius'])),iw*0.5,ih*0.5)
                        tl=tr=br=bl=rv
                    x0,y0,x1,y1=ins,ins,sw_dp-ins,sh_dp-ins
                    pdata=f'M{x0+tl},{y0} L{x1-tr},{y0} A{tr},{tr} 0 0,1 {x1},{y0+tr} L{x1},{y1-br} A{br},{br} 0 0,1 {x1-br},{y1} L{x0+bl},{y1} A{bl},{bl} 0 0,1 {x0},{y1-bl} L{x0},{y0+tl} A{tl},{tl} 0 0,1 {x0+tl},{y0} Z'
                pattrs={A+'pathData':pdata}
                if sd['bg_rgba'] is not None:
                    r,g,b,al=sd['bg_rgba']
                    pattrs[A+'fillColor']=f'#{r:02x}{g:02x}{b:02x}' if al==255 else f'#{al:02x}{r:02x}{g:02x}{b:02x}'
                if sd['stroke_rgba'] is not None and stw>0:
                    r,g,b,al=sd['stroke_rgba']
                    pattrs[A+'strokeColor']=f'#{r:02x}{g:02x}{b:02x}' if al==255 else f'#{al:02x}{r:02x}{g:02x}{b:02x}'
                    pattrs[A+'strokeWidth']=str(stw)
                ET.SubElement(vec,'path',pattrs)
                with tempfile.TemporaryDirectory() as td:
                    tmp_vec=Path(td)/'shape_vec.xml'
                    ET.ElementTree(vec).write(tmp_vec,encoding='utf-8',xml_declaration=True)
                    if target_name not in images:images[target_name]=rasterize_vector(tmp_vec,density)
                return (True,'',False)
            return (False,'',False)
        def resolve_image_drawable(dref, target_name, seen=(), in_selector=False):
            if dref in seen:raise ValueError('Cyclic drawable reference '+dref)
            if seen and dref=='@null':return (True,'',False)
            if seen and not dref.startswith('@drawable/'):
                r,g,b,al=static_color_rgba(dref)
                if target_name not in images:
                    px_sz=max(1,int(round(24.0*density/160.0)))
                    images[target_name]=(px_sz,px_sz,bytes([r,g,b,al])*(px_sz*px_sz),160.0/density)
                return (True,'',False)
            dname=dref.split('/')[-1]
            candidates,nodpi,xml_file=find_drawable_candidates(dname)
            if len({dpi for dpi,_ in candidates}) != len(candidates):raise ValueError('Ambiguous PNG drawable density '+dref)
            image_file=None;source_density=160
            if candidates:
                higher=sorted((dpi,file) for dpi,file in candidates if dpi>=density)
                source_density,image_file=higher[0] if higher else max(candidates)
            elif nodpi is not None:
                image_file=nodpi
            if image_file is not None:
                w,h,pixels,scale,_,stretch_spec=decode_drawable_image(image_file,source_density)
                if target_name not in images:
                    images[target_name]=(w,h,pixels,scale)
                return (True,stretch_spec,False)
            if xml_file is not None:
                return resolve_image_element(ET.parse(xml_file).getroot(),dref,target_name,seen,xml_file=xml_file,in_selector=in_selector)
            return (False,'',False)
        if n.tag in {'ImageView','ImageButton','ZoomButton','Toolbar','QuickContactBadge'} and src and resource_roots:
            name=src.split('/')[-1]
            img_ok,img_st,img_am=resolve_image_drawable(src,name)
            if not img_ok:
                if n.tag in {'ImageView','QuickContactBadge'} or name not in icons:
                    raise ValueError('Unresolved PNG drawable '+src)
            else:
                src_auto_mirrored=img_am
                if n.tag in {'ImageView','ImageButton','ZoomButton','QuickContactBadge'}:
                    if src_stateful_images:
                        for req,forbid,ik,ist in src_stateful_images:
                            relative.append('RelativeRule { kind:253, anchor:'+json.dumps(f'{req},{forbid}|{ik}|{ist}')+' }')
                    elif img_st:
                        relative.append('RelativeRule { kind:252, anchor:'+json.dumps(img_st)+' }')
                    if src_animation_list is not None:
                        relative.append('RelativeRule { kind:285, anchor:'+json.dumps(src_animation_list)+' }')
                    if src_dynamic_drawable is not None:
                        relative.append('RelativeRule { kind:286, anchor:'+json.dumps('src:'+src_dynamic_drawable)+' }')
                    for trans_spec in src_animated_transitions:
                        relative.append('RelativeRule { kind:293, anchor:'+json.dumps('src:'+trans_spec)+' }')
                    if src_animated_vector is not None:
                        relative.append('RelativeRule { kind:309, anchor:'+json.dumps('src:'+src_animated_vector)+' }')
        text=a.get('text','') if n.tag in {'EditText','AutoCompleteTextView','MultiAutoCompleteTextView'} else a.get('text',a.get('hint',a.get('contentDescription','')))
        hint=a.get('hint','') if n.tag in {'EditText','AutoCompleteTextView','MultiAutoCompleteTextView'} else ''
        if n.tag=='Toolbar':text=a.get('title','');hint=a.get('subtitle','')
        base=a.get('layout_margin','0dp');horizontal=a.get('layout_marginHorizontal',base);vertical=a.get('layout_marginVertical',base)
        margin=[a.get('layout_marginLeft',horizontal),a.get('layout_marginTop',vertical),a.get('layout_marginRight',horizontal),a.get('layout_marginBottom',vertical)]
        margin='['+','.join(margin_number(v) for v in margin)+']'
        margin_relative='['+','.join('Some('+margin_number(a[k])+')' if k in a else 'None' for k in ('layout_marginStart','layout_marginEnd'))+']'
        pad=[a.get('padding',a.get(axis,a.get(side,'0dp'))) for axis,side in [('paddingHorizontal','paddingLeft'),('paddingVertical','paddingTop'),('paddingHorizontal','paddingRight'),('paddingVertical','paddingBottom')]]
        padding_edges='['+','.join(number(v) for v in pad)+']'
        padding_explicit='['+','.join(str('padding' in a or axis in a or side in a).lower() for axis,side in [('paddingHorizontal','paddingLeft'),('paddingVertical','paddingTop'),('paddingHorizontal','paddingRight'),('paddingVertical','paddingBottom')])+']'
        padding_relative='['+','.join('Some('+number(a[k])+')' if k in a else 'None' for k in ('paddingStart','paddingEnd'))+']'
        bg_d=background(a['background']) if 'background' in a and not background_selector else empty_drawable_dict()
        bg=bg_d['bg'];radius=bg_d['radius'];bg_stroke=bg_d['stroke'];bg_width=bg_d['stroke_width']
        bg_padding='Some(['+','.join(str(x) for x in bg_d['padding'])+'])' if bg_d['padding'] is not None else 'None'
        bg_min='['+','.join(str(x) for x in bg_d['size'])+']'
        emitted_corner_radii=False
        if bg_d['corner_radii'] is not None:
            relative.append('RelativeRule { kind:206, anchor:'+json.dumps(bg_d['corner_radii'])+' }')
            emitted_corner_radii=True
        if bg_d['gradient'] is not None:relative.append('RelativeRule { kind:207, anchor:'+json.dumps(bg_d['gradient'])+' }')
        for layer in bg_d['layers']:relative.append('RelativeRule { kind:208, anchor:'+json.dumps(format_layer_rule(layer))+' }')
        has_ripple_rule=False
        if bg_d['ripple'] is not None:
            relative.append('RelativeRule { kind:209, anchor:'+json.dumps(bg_d['ripple'])+' }')
            has_ripple_rule=True
        if bg_d['bg_selector']:
            for rule in bg_d['bg_selector']:relative.append('RelativeRule { kind:96, anchor:'+json.dumps(rule)+' }')
        for req,forbid,layer in bg_d['stateful_layers']:
            relative.append('RelativeRule { kind:231, anchor:'+json.dumps(f'{req},{forbid}|'+format_layer_rule(layer))+' }')
        if bg_d['stateful_bg_images']:
            for req,forbid,img_k,img_st in bg_d['stateful_bg_images']:
                relative.append('RelativeRule { kind:254, anchor:'+json.dumps(f'{req},{forbid}|{img_k}|{img_st}')+' }')
        elif bg_d['bg_image'] is not None:
            img_k,img_st=bg_d['bg_image']
            relative.append('RelativeRule { kind:251, anchor:'+json.dumps(f'{img_k}|{img_st}')+' }')
        if bg_d['animation_list'] is not None:
            relative.append('RelativeRule { kind:285, anchor:'+json.dumps(bg_d['animation_list'])+' }')
        if bg_d['dynamic_drawable'] is not None:
            relative.append('RelativeRule { kind:286, anchor:'+json.dumps('bg:'+bg_d['dynamic_drawable'])+' }')
        for trans_spec in bg_d['animated_transitions']:
            relative.append('RelativeRule { kind:293, anchor:'+json.dumps('bg:'+trans_spec)+' }')
        if bg_d['animated_vector'] is not None:
            relative.append('RelativeRule { kind:309, anchor:'+json.dumps('bg:'+bg_d['animated_vector'])+' }')
        if src_auto_mirrored or bg_d['auto_mirrored'] or compound_auto_mirrored:
            relative.append('RelativeRule { kind:255, anchor:"true" }')
        if bg_d['bitmap_tint'] is not None and 'backgroundTint' not in a:
            relative.append('RelativeRule { kind:194, anchor:'+json.dumps(rgba_to_csv(bg_d['bitmap_tint']))+' }')
        shape_applied=False
        shape_corner_radii=None
        shape_radius=None
        shape_corner_families=None
        direct_shape={sk:n.attrib[APP+sk] for sk in SHAPE_APPEARANCE_KEYS if APP+sk in n.attrib}
        if APP+'shapeAppearance' in n.attrib or APP+'shapeAppearanceOverlay' in n.attrib or direct_shape:
            fixed_dims=[]
            for dim_k in ('layout_width','layout_height'):
                raw_d=a.get(dim_k,'')
                if raw_d and raw_d not in {'match_parent','fill_parent','wrap_content'}:
                    try:
                        dv=float(number(raw_d))
                        if dv>0.0:fixed_dims.append(dv)
                    except ValueError:
                        pass
            ref_dp=min(fixed_dims) if fixed_dims else 48.0
            def parse_corner_val(raw):
                rv=resolve(raw).strip()
                tok=theme_token(rv)
                if tok is not None:
                    if tok not in THEME_DIMENS:raise ValueError('Unresolved theme dimension '+rv)
                    rv=THEME_DIMENS[tok]
                if rv.endswith('%'):
                    try: pct=float(rv[:-1])
                    except ValueError: raise ValueError('Invalid cornerSize '+raw)
                    if not (0.0<=pct<=100.0) or pct!=pct:raise ValueError('Invalid cornerSize '+raw)
                    if pct>=50.0:return ('pill',pct)
                    if pct==0.0:return ('dp',0.0)
                    return ('dp',round(ref_dp*pct/100.0,4))
                return ('dp',float(number(rv)))
            def parse_corner_family(raw):
                rv=resolve(raw).strip()
                if theme_token(rv)=='shapeCornerFamily':return 'r'
                if rv in {'rounded','0'}:return 'r'
                if rv in {'cut','1'}:return 'c'
                raise ValueError('Unsupported cornerFamily '+raw)
            corners_state=[None,None,None,None]
            families_state=['r','r','r','r']
            shape_layers=[]
            if APP+'shapeAppearance' in n.attrib:
                shape_layers.extend(resolve_shape_style_layers(n.attrib[APP+'shapeAppearance']))
            if APP+'shapeAppearanceOverlay' in n.attrib:
                shape_layers.extend(resolve_shape_style_layers(n.attrib[APP+'shapeAppearanceOverlay']))
            if direct_shape:
                shape_layers.append(direct_shape)
            for layer_dict in shape_layers:
                if 'cornerFamily' in layer_dict:
                    cf=parse_corner_family(layer_dict['cornerFamily'])
                    families_state=[cf,cf,cf,cf]
                for idx,fk in enumerate(('cornerFamilyTopLeft','cornerFamilyTopRight','cornerFamilyBottomRight','cornerFamilyBottomLeft')):
                    if fk in layer_dict:
                        families_state[idx]=parse_corner_family(layer_dict[fk])
                if 'cornerSize' in layer_dict:
                    cv=parse_corner_val(layer_dict['cornerSize'])
                    corners_state=[cv,cv,cv,cv]
                for idx,ck in enumerate(('cornerSizeTopLeft','cornerSizeTopRight','cornerSizeBottomRight','cornerSizeBottomLeft')):
                    if ck in layer_dict:
                        corners_state[idx]=parse_corner_val(layer_dict[ck])
            if any(f=='c' for f in families_state):
                shape_corner_families=','.join(families_state)
            if any(c is not None for c in corners_state):
                shape_applied=True
                filled=[c if c is not None else ('dp',0.0) for c in corners_state]
                if all(kind=='pill' for kind,_ in filled):
                    shape_radius='-1.0'
                else:
                    vals=[(round(ref_dp*0.5,4) if fixed_dims else 9999.0) if kind=='pill' else v for kind,v in filled]
                    if vals[0]==vals[1]==vals[2]==vals[3]:
                        shape_radius=str(vals[0])
                    else:
                        shape_radius='0.0'
                        shape_corner_radii=f'{vals[0]},{vals[1]},{vals[2]},{vals[3]}'
        if bg=='None' and not background_selector and not bg_d['bg_selector'] and not bg_d['layers'] and not bg_d['stateful_layers'] and bg_d['gradient'] is None and bg_d['bg_image'] is None and not bg_d['stateful_bg_images'] and 'backgroundTint' in a:
            bg=rgba_to_rust(static_color_rgba(a['backgroundTint']))
        if n in extra_strokes:
            sc_raw,sw_raw=extra_strokes[n]
            if sc_raw is not None and bg_stroke=='None' and resolve(sc_raw)!='@null':bg_stroke=rgba_to_rust(static_color_rgba(sc_raw))
            if sw_raw is not None and bg_width=='0.0':bg_width=number(sw_raw)
        has_explicit_button_corner=n.tag=='Button' and APP+'cornerRadius' in n.attrib
        if shape_applied and not has_explicit_button_corner and bg_d['corner_radii'] is None and radius=='0.0':
            if shape_corner_radii is not None:
                relative.append('RelativeRule { kind:206, anchor:'+json.dumps(shape_corner_radii)+' }')
                emitted_corner_radii=True
            elif shape_radius is not None:
                radius=shape_radius
        if shape_corner_families is not None:
            relative.append('RelativeRule { kind:278, anchor:'+json.dumps(shape_corner_families)+' }')
        if n in track_corner_radii and radius=='0.0' and bg_d['corner_radii'] is None and shape_corner_radii is None:
            radius=track_corner_radii[n]
        if n.tag=='Button':
            if APP+'cornerRadius' in n.attrib and radius=='0.0':radius=number(n.attrib[APP+'cornerRadius'])
            if APP+'strokeColor' in n.attrib and bg_stroke=='None' and resolve(n.attrib[APP+'strokeColor'])!='@null':bg_stroke=rgba_to_rust(static_color_rgba(n.attrib[APP+'strokeColor']))
            if APP+'strokeWidth' in n.attrib and bg_width=='0.0':bg_width=number(n.attrib[APP+'strokeWidth'])
            if APP+'rippleColor' in n.attrib and not has_ripple_rule:
                relative.append('RelativeRule { kind:209, anchor:'+json.dumps(rgba_to_csv(static_color_rgba(n.attrib[APP+'rippleColor'])))+' }')
                has_ripple_rule=True
        if n in fab_nodes and 'background' not in a:
            if not shape_applied:radius='-1.0'
            if bg=='None':bg=color('?attr/colorPrimaryContainer')
        if n in extended_fab_specs and 'background' not in a:
            if not shape_applied and not has_explicit_button_corner and radius=='0.0':radius='16.0'
            if bg=='None':bg=color('?attr/colorPrimaryContainer')
        if n in chip_nodes and 'background' not in a:
            if not shape_applied and radius=='0.0':radius='16.0'
            if bg=='None':bg=color('?attr/colorSurfaceContainerLow')
            if bg_stroke=='None':bg_stroke=color('?attr/colorOutline');bg_width='1.0'
        fg_d=resolve_drawable_ref(a['foreground'],allow_dynamic=False) if 'foreground' in a else empty_drawable_dict()
        fg=fg_d['bg'];fg_radius=fg_d['radius'];fg_stroke=fg_d['stroke'];fg_width=fg_d['stroke_width']
        fg_padding='Some(['+','.join(str(x) for x in fg_d['padding'])+'])' if fg_d['padding'] is not None else 'None'
        fg_size='['+','.join(str(x) for x in fg_d['size'])+']'
        if fg_d['ripple'] is not None and not has_ripple_rule:
            relative.append('RelativeRule { kind:209, anchor:'+json.dumps(fg_d['ripple'])+' }')
            has_ripple_rule=True
        clip_outline=a.get('clipToOutline')
        if clip_outline=='true' or (clip_outline!='false' and (material_card or n in shapeable_image_nodes or n in carousel_item_nodes) and (float(radius)!=0.0 or emitted_corner_radii)):
            relative.append('RelativeRule { kind:234, anchor:"true" }')
        relative_rules='&['+','.join(relative)+']'
        if n.tag=='GridLayout':
            for child in n:
                for attr,limit in [('layout_column','columnCount'),('layout_row','rowCount')]:
                    raw=child.attrib.get(A+attr)
                    span_key='layout_columnSpan' if attr=='layout_column' else 'layout_rowSpan'
                    span=child.attrib.get(A+span_key,'1')
                    if limit in a and re.fullmatch(r'\d+',resolve(span)):
                        start=int(resolve(raw)) if raw is not None and re.fullmatch(r'\d+',resolve(raw)) else 0
                        if start+int(resolve(span))>int(a[limit]):raise ValueError(attr+' exceeds '+limit)
        children=',\n'.join(node(c,n.tag,n) for c in n)
        if n.tag in {'ScrollView','HorizontalScrollView'} and len(n)!=1:raise ValueError(n.tag+' requires exactly one child')
        if n.tag in {'ViewSwitcher','TextSwitcher','ImageSwitcher'} and len(n)>2:raise ValueError('ViewSwitcher supports at most two children')
        if n.tag=='TextSwitcher' and any(c.tag!='TextView' for c in n):raise ValueError('TextSwitcher children must be TextView')
        if n.tag=='ImageSwitcher' and any(c.tag!='ImageView' for c in n):raise ValueError('ImageSwitcher children must be ImageView')
        if len(n) and n.tag not in CONTAINER_TAGS:raise ValueError('Leaf view cannot have children')
        resolved_img_tint=color(a['tint']) if 'tint' in a and tint_states is None else (vec_default_tint if tint_states is None else 'None')
        resolved_img_tint_mode={'src_atop':0,'src_in':1,'src_over':2,'multiply':3,'screen':4,'add':5}.get(a['tintMode'],0) if 'tintMode' in a else (vec_default_tint_mode if ('tint' not in a and vec_default_tint!='None') else 0)
        return '''Node {kind:Kind::%s,id:%s,width:%s,height:%s,min_width:%s,min_height:%s,horizontal:%s,measure_all_children:%s,layout_direction:%s,fill_viewport:%s,clip_to_padding_disabled:%s,clip_children_disabled:%s,weight:%s,weight_sum:%s,divider_color:%s,divider_thickness:%s,divider_flags:%s,divider_padding:%s,padding_edges:%s,padding_explicit:%s,background_padding:%s,background_min_size:%s,padding_relative:%s,margin:%s,margin_relative:%s,layout_gravity:[%s,%s],absolute_position:[%s,%s],relative_rules:%s,layout_gravity_explicit:%s,content_gravity:[%s,%s],text:%s,hint:%s,chrono_format:%s,count_down:%s,clock_12:%s,clock_24:%s,input_type:%s,text_on:%s,text_off:%s,text_size:%s,letter_spacing_em:%s,line_spacing_extra:%s,line_spacing_multiplier:%s,text_bold:%s,text_all_caps:%s,max_lines:%s,min_lines:%s,ellipsize_end:%s,ellipsize_side:%s,single_line:%s,align:%s,color:%s,text_color_states:%s,background:%s,foreground:%s,foreground_radius:%s,foreground_size:%s,foreground_padding:%s,foreground_gravity:[%s,%s],foreground_excludes_padding:%s,background_stroke:%s,background_stroke_width:%s,foreground_stroke:%s,foreground_stroke_width:%s,radius:%s,clickable:%s,icon:MaterialIcon::%s,image_src:%s,image_scale:%s,image_adjust_bounds:%s,image_max_width:%s,image_max_height:%s,image_tint:%s,image_tint_states:%s,image_tint_mode:%s,image_alpha:%s,view_alpha:%s,image_builtin:%s,enabled:%s,checked:%s,checked_button:%s,min:%s,max:%s,progress:%s,secondary_progress:%s,indeterminate:%s,num_stars:%s,rating:%s,step_size:%s,indicator:%s,mirror_for_rtl:%s,animation_in:%s,animation_out:%s,translate_in:%s,translate_out:%s,auto_start:%s,flip_interval:%s,visibility:Visibility::%s,children:&[%s]}'''%(TAGS[n.tag],json.dumps(id),size(a.get('layout_width','wrap_content')),size(a.get('layout_height','wrap_content')),number(a.get('minWidth','0')),number(a.get('minHeight','0')),str(orientation=='horizontal').lower(),str(a.get('measureAllChildren','true' if n.tag in {'ViewAnimator','ViewSwitcher','TextSwitcher','ImageSwitcher','ViewFlipper'} else 'false')=='true').lower(),{'inherit':0,'ltr':1,'rtl':2}[direction],str(a.get('fillViewport','false')=='true').lower(),str(a.get('clipToPadding','true')=='false').lower(),str(a.get('clipChildren','true')=='false').lower(),number(a.get('layout_weight','0')),number(a.get('weightSum','0')),divider_color,divider_thickness,divider_flags,number(a.get('dividerPadding','0')),padding_edges,padding_explicit,bg_padding,bg_min,padding_relative,margin,margin_relative,horizontal_gravity,vertical_gravity,margin_number(a.get('layout_x','0')),margin_number(a.get('layout_y','0')),relative_rules,str('layout_gravity' in a).lower(),content_h,content_v,json.dumps(text,ensure_ascii=False),json.dumps(hint,ensure_ascii=False),json.dumps(chrono_format,ensure_ascii=False),str(a.get('countDown','false')=='true').lower(),json.dumps(a.get('format12Hour',''),ensure_ascii=False),json.dumps(a.get('format24Hour',''),ensure_ascii=False),input_type_code,json.dumps(a.get('textOn','ON'),ensure_ascii=False),json.dumps(a.get('textOff','OFF'),ensure_ascii=False),number(a.get('textSize','16sp')),str(float(letter_spacing)),margin_number(a.get('lineSpacingExtra','0dp')),str(float(spacing_multiplier)),str(text_is_bold).lower(),str(a.get('textAllCaps','false')=='true').lower(),int(max_lines),int(min_lines),str(ellipsize in {'end','marquee'}).lower(),{'start':1,'middle':2}.get(ellipsize,0),str(single_line).lower(),1 if content_h==1 else 2 if content_h in (2,4) else 0,color(a['textColor']) if 'textColor'in a and text_color_states is None else 'None',text_color_states or '&[]',bg,fg,fg_radius,fg_size,fg_padding,fg_h,fg_v,str(a.get('foregroundInsidePadding','true')=='false').lower(),bg_stroke,bg_width,fg_stroke,fg_width,radius,str(a.get('clickable','false')=='true').lower(),icons.get(icon,'Menu'),json.dumps(src[10:] if src.startswith('@drawable/') else ''),{'fitCenter':0,'centerCrop':1,'fitXY':2,'center':3,'centerInside':4,'fitStart':5,'fitEnd':6,'matrix':7}.get(scale_type,0),str(a.get('adjustViewBounds','false')=='true').lower(),('Some('+number(a['maxWidth'])+')' if 'maxWidth' in a else 'None'),('Some('+number(a['maxHeight'])+')' if 'maxHeight' in a else 'None'),resolved_img_tint,tint_states or '&[]',resolved_img_tint_mode,int(image_alpha),str(float(view_alpha)),str(icon in icons).lower(),str(a.get('enabled','true')!='false').lower(),str(a.get('checked','false')=='true').lower(),json.dumps(checked_button),str(float(minimum)),str(float(maximum)),str(float(progress)),str(float(secondary)),str(a.get('indeterminate','false' if orientation=='horizontal' else 'true')=='true').lower(),str(stars),str(rating),str(step),str(a.get('isIndicator','false')=='true').lower(),str(a.get('mirrorForRtl','false')=='true').lower(),anim_in,anim_out,translate_in,translate_out,str(a.get('autoStart','false')=='true').lower(),str(int(interval)/1000.0),visibility.capitalize(),children)
    resolved_merge_parent=None
    if merge_parent is not None:
        resolved_merge_parent=canonical_view_tag(merge_parent)
        if resolved_merge_parent not in CONTAINER_TAGS:
            raise ValueError('Unsupported merge parent '+str(merge_parent))
    path_obj=Path(path)
    load_path=path_obj
    if path_obj.parent.name=='layout' and any(path_obj.parent.parent.resolve()==r.resolve() for r in resource_roots):
        q_layout=find_resource_file('layout',path_obj.name)
        if q_layout is not None:load_path=q_layout
    root=ET.parse(load_path).getroot()
    if resolved_merge_parent is not None:
        unwrapped=unwrap_databinding(root)
        if unwrapped.tag=='merge':
            wrap_attrib={A+'layout_width':unwrapped.attrib.get(A+'layout_width','match_parent'),A+'layout_height':unwrapped.attrib.get(A+'layout_height','match_parent')}
            for keep_k in (A+'id',A+'visibility'):
                if keep_k in unwrapped.attrib:wrap_attrib[keep_k]=unwrapped.attrib[keep_k]
            if resolved_merge_parent in {'LinearLayout','RadioGroup','GridLayout'} and A+'orientation' in unwrapped.attrib:
                wrap_attrib[A+'orientation']=unwrapped.attrib[A+'orientation']
            wrapper=ET.Element(resolved_merge_parent,wrap_attrib)
            if merge_parent in {'androidx.coordinatorlayout.widget.CoordinatorLayout','CoordinatorLayout'}:
                coordinator_nodes.add(wrapper)
            if merge_parent=='com.google.android.material.appbar.CollapsingToolbarLayout':
                ctl_h='152'
                raw_ctl_h=resolve(wrap_attrib.get(A+'layout_height','match_parent')).strip()
                if raw_ctl_h not in {'match_parent','fill_parent','wrap_content'}:
                    hv=number(raw_ctl_h)
                    if float(hv)>0.0:ctl_h=hv
                collapsing_toolbar_specs[wrapper]=f'|{ctl_h}|'
            if merge_parent=='com.google.android.material.search.SearchView':
                search_view_ext_specs[wrapper]='||0|0'
            if merge_parent in {'androidx.swiperefreshlayout.widget.SwipeRefreshLayout','android.support.v4.widget.SwipeRefreshLayout'}:
                swipe_refresh_nodes.add(wrapper)
            if merge_parent=='androidx.drawerlayout.widget.DrawerLayout':
                drawer_layout_nodes.add(wrapper)
            if merge_parent in {'androidx.viewpager2.widget.ViewPager2','androidx.viewpager.widget.ViewPager','android.support.v4.view.ViewPager'}:
                vp_orient=resolve(unwrapped.attrib.get(A+'orientation','horizontal')).strip()
                if vp_orient not in {'horizontal','vertical'}:raise ValueError('Unsupported ViewPager orientation '+vp_orient)
                wrapper.attrib[A+'orientation']=vp_orient
                view_pager_specs[wrapper]=vp_orient
                if vp_orient=='vertical':
                    view_pager_ext_specs[wrapper]='v|1|0|none'
            if merge_parent=='androidx.appcompat.widget.AlertDialogLayout':
                alert_dialog_specs[wrapper]=''
            if merge_parent=='com.google.android.material.snackbar.SnackbarContentLayout':
                if A+'orientation' not in wrapper.attrib:wrapper.attrib[A+'orientation']='horizontal'
                snackbar_nodes.add(wrapper)
            if merge_parent=='com.google.android.material.floatingtoolbar.FloatingToolbarLayout':
                if A+'orientation' not in wrapper.attrib:wrapper.attrib[A+'orientation']='horizontal'
                floating_toolbar_nodes.add(wrapper)
            if merge_parent=='com.google.android.material.button.MaterialButtonGroup':
                if A+'orientation' not in wrapper.attrib:wrapper.attrib[A+'orientation']='horizontal'
                button_group_specs[wrapper]='12.0'
            if merge_parent=='com.google.android.material.button.MaterialSplitButton':
                if A+'orientation' not in wrapper.attrib:wrapper.attrib[A+'orientation']='horizontal'
                split_button_nodes.add(wrapper)
            if merge_parent=='com.google.android.material.carousel.MaskableFrameLayout':
                carousel_item_nodes.add(wrapper)
            if merge_parent=='com.google.android.material.chip.ChipGroup':
                if A+'orientation' not in wrapper.attrib:wrapper.attrib[A+'orientation']='horizontal'
                chip_group_specs[wrapper]='0|0|8.0|8.0'
            if merge_parent=='com.google.android.material.tabs.TabLayout':
                tab_options_specs[wrapper]='fixed|primary|'
            for child in list(unwrapped):wrapper.append(child)
            root=wrapper
    initial=path_obj.stem if any(path_obj.parent.parent.resolve()==r.resolve() and (path_obj.parent.name=='layout' or path_obj.parent.name.startswith('layout-')) for r in resource_roots) else ''
    expanded=expand(root,(initial,) if initial else ())
    if len(expanded)!=1:raise ValueError('A layout must have exactly one root')
    root=expanded[0]
    validate_relative_cycles(root)
    result='// Generated by tools/import-android-layout.py. Edit the XML source.\n#[allow(unused_imports)]\nuse bhoo::{android_xml::*,Color,MaterialIcon};\npub static LAYOUT:Node='+node(root)+';\n'
    if images:
        result+='pub fn install_drawables(state: &mut LayoutState) {\n'
        for name,tint in indicator_tints.items():
            if tint=='controlNormal':value='AndroidDrawableTint::ControlNormal'
            else:
                alpha,red,green,blue=(int(tint[i:i+2],16) for i in (1,3,5,7))
                value=f'AndroidDrawableTint::Fixed(Color({red},{green},{blue},{alpha}))'
            result+='  state.drawable_tints.insert('+json.dumps(name)+'.into(),'+value+');\n'
        for name,(width,height,pixels,scale) in images.items():
            result+='  state.image_scales.insert('+json.dumps(name)+'.into(),'+str(scale)+');\n'
            result+='  state.images.entry('+json.dumps(name)+'.into()).or_insert_with(|| bhoo::Image::rgba('+str(width)+','+str(height)+',vec!['+','.join(map(str,pixels))+']).expect("validated drawable"));\n'
        result+='}\n'
    return result
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('input',type=Path);parser.add_argument('output',type=Path);parser.add_argument('--resources',type=Path,action='append');parser.add_argument('--density',type=int,default=160);parser.add_argument('--night',action='store_true');parser.add_argument('--orientation',choices=('port','land'),default='port');parser.add_argument('--sw',type=int,default=0);parser.add_argument('--locale',default=None);dir_group=parser.add_mutually_exclusive_group();dir_group.add_argument('--ldrtl',action='store_true');dir_group.add_argument('--ldltr',action='store_true');parser.add_argument('--merge-parent',default=None);parser.add_argument('--custom-view-map',type=Path,default=None);args=parser.parse_args()
    cli_ldrtl=True if args.ldrtl else (False if args.ldltr else None)
    try:output=compile_layout(args.input,args.resources,args.density,night=args.night,orientation=args.orientation,sw=args.sw,merge_parent=args.merge_parent,custom_view_map=args.custom_view_map,locale=args.locale,ldrtl=cli_ldrtl)
    except (ValueError,ET.ParseError,OSError,json.JSONDecodeError) as e:parser.exit(1,str(e)+'\n')
    args.output.write_text(output)
