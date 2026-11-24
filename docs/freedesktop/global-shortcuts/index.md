[Jump to content](#_content)[Global Shortcuts Specification](#id-1)[Show Contents: Global Shortcuts Specification](#id-1)[Top](#)
1. [1 About](#about)
2. [2 Required Rules](#required-rules)
3. [3 Recommended Rules](#recommended-rules)

## Global Shortcuts Specification [#](#id-1)

- File Name: global-shortcuts-spec.xml
- ID: no ID found

Author: Noah Davis `<noahadvs@gmail.com>`
Publication Date: 22 May 2020, Version: 0.1 (DRAFT)

- **[1 About](#about)**
- **[2 Required Rules](#required-rules)**
- **[3 Recommended Rules](#recommended-rules)**

## 1 About [#](#about)

- File Name: global-shortcuts-spec.xml
- ID: about

 The purpose of this specification is to reduce the amount of situations where an application and a desktop environment or window manager could have conflicting shortcuts. It is currently intended to be small and easy to comply with. In the future, it could have rules and exceptions for specific shortcuts added, but specific shortcuts are merely examples for now.

 For the sake of brevity, desktop environment and window manager shortcuts will be called global shortcuts in this specification since they usually have a global scope.

## 2 Required Rules [#](#required-rules)

- File Name: global-shortcuts-spec.xml
- ID: required-rules

 These are required rules because they are needed for this specification to work. They should be generally easy to comply with.

1.

 Applications should never use the Meta/Super key.

## 3 Recommended Rules [#](#recommended-rules)

- File Name: global-shortcuts-spec.xml
- ID: recommended-rules

 These are recommended because a global shortcut may need to be an exception to the rule.

1.

 Global shortcuts should use the Meta/Super key.

  -

 Examples:

    -

 Many creative workflow apps use Alt + Left Click for certain actions. However, global shortcuts for moving windows sometimes conflict with those actions. Instead of Alt + Left Click, those global shortcuts should use Meta/Super + Left Click.

  -

 Examples of possible exceptions:

    -

 Many users expect Alt + Tab to switch windows. Most applications already avoid using Alt + Tab for compatibility with existing global shortcuts and cross platform compatibility.

© 2026 Freedesktop.org
