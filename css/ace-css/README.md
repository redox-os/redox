
# ace-css

Full-featured [Basscss](http://basscss.com) package for web apps

ace-css contains the core modules and several addons for use in larger web apps.

## Features

- All typography and layout utilities from the core Basscss module
- Base element styles from basscss-basic
- Form styles from basscss-forms
- Button styles with solid and outline styles, along with size modifiers
- Range input styles from basscss-input-range
- Progress element styles from basscss-progress
- Responsive margin and padding utilities
- Media Object
- Colors, background colors, and border colors from [colors.css](http://clrs.cc)
- Darken and lighten background utilities
- Background image utilities

## Install

```sh
npm i ace-css
```

## Usage

### PostCSS

```css
@import 'ace-css';
```

### Webpack

```js
import aceCss from 'ace-css/css/ace.min.css'
```

[MIT License](LICENSE.md)
