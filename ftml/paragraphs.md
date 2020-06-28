(temporary documentation until this exists in code)

Wikidot inserts `<br>`s whenever a newline is present in the source,
however two newlines next to each other mark the beginning of a new paragraph.

Consider this sample code:
```
A
B

C

D
E
F

G

H

I
```

From this, the following HTML is produced:
```html
<p>
  A <br>
  B
</p>

<p>
  C
</p>

<p>
  D <br>
  E <br>
  F
</p>

<p>
  G
</p>

<p>
  H
</p>

<p>
  I
</p>
```

If we escape the initial source's newlines, this can be thought of as a substitution:
```
A . B . . C . . D . E . F . . G . . H . . I
```

Where `.` (one newline) becomes `<br>` and `. .` (two newlines) becomes `</p> <p>`:

```html
A <br> B </p> <p> C </p> <p> D <br> E <br> F </p> <p> G </p> <p> H </p> <p> I
```

Which, when formatted ad missing tags are added:

```html
<p>
  A <br>
  B
</p>

<p>
  C
</p>

<p>
  D <br>
  E <br>
  F
</p>

<p>
  G
</p>

<p>
  H
</p>

<p>
  I
</p>
```

Which matches what was generated.
