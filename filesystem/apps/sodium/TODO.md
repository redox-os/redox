- [x] Make editor.pos method and use that instead of
- [ ] Add word navigation


Known bugs:

- [x] When using `t` with a char that isn't in the document, Sodium will crash.
- [x] `d<motion>` does not do anything if: 1) the motion moves to the end of a line. 2) if the motion moves to the last line.

The bug causing these two bugs, is localised to be in position.rs. It resolves by returning a value one over bound x

- [x] The x value is wrongly bounded. Reproduction:
      1) Make two lines:
         - abc
         - abcdef
      2) Go to the end of the first line.
      3) Go one down. As you'll see you'll end up at d. That's right.
      4) Now go two the end of the first line again.
      5) Type 2l.
      6) Now go one down
      7) You'll end up on e, even though it should be d

- [x] Crashes when:
      1) Write abc on line 1
      2) Press o to go to the next line
      3) Go to normal mode
      4) Press a and go to append mode
      5) Type text
      6) Out of bound (index) error

- [x] When typing the first char in a line in normal insert mode, it wont go to the next char.

- [ ] The modifier keys are only working for one command
