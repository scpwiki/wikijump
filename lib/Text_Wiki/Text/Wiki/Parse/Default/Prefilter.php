<?php

/**
*
* "Pre-filter" the source text.
*
* @category Text
*
* @package Text_Wiki
*
* @author Paul M. Jones <pmjones@php.net>
*
* @license LGPL
*
* @version $Id$
*
*/

/**
*
* "Pre-filter" the source text.
*
* Convert DOS and Mac line endings to Unix, concat lines ending in a
* backslash \ with the next line, convert tabs to 4-spaces, add newlines
* to the top and end of the source text, compress 3 or more newlines to
* 2 newlines.
*
* @category Text
*
* @package Text_Wiki
*
* @author Paul M. Jones <pmjones@php.net>
*
*/

class Text_Wiki_Parse_Prefilter extends Text_Wiki_Parse {

    /**
    *
    * Simple parsing method.
    *
    * @access public
    *
    */

    function parse()
    {
        // convert DOS line endings
        $this->wiki->source = str_replace("\r\n", "\n",
            $this->wiki->source);

        // convert Macintosh line endings
        $this->wiki->source = str_replace("\r", "\n",
            $this->wiki->source);

        // trim "whitespace lines"
        $this->wiki->source = preg_replace("/^\s+$/m", '', $this->wiki->source);

       // // concat lines ending in a backslash
       // $this->wiki->source = str_replace("\\\n", "",

        // convert tabs to four-spaces
        $this->wiki->source = str_replace("\t", "    ",
            $this->wiki->source);

        // add extra newlines at the top and end; this
        // seems to help many rules.
        $this->wiki->source =  "\n".$this->wiki->source . "\n\n";
        // finally, compress all instances of 3 or more newlines
        // down to two newlines.
        $find = "/(\n[ ]*){3,}/m";
        $replace = "\n\n";
        $this->wiki->source = preg_replace($find, $replace,
            $this->wiki->source);

        $d = utf8_encode("\xFC");
        $this->wiki->source = str_replace("$d$d", '', $this->wiki->source);
        // TODO: sth more intelligent?Prefilter.php
    }

}
