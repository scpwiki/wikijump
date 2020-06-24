<?php

/**
*
* Parse for URLS in the source text.
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
* Parse for URLS in the source text.
*
* Various URL markings are supported: inline (the URL by itself),
* numbered or footnote reference (where the URL is enclosed in square
* brackets), and named reference (where the URL is enclosed in square
* brackets and has a name included inside the brackets).  E.g.:
*
* inline    -- http://example.com or www.example.com
* numbered  -- [http://example.com]
* described -- [http://example.com Example Description]
*
* When rendering a URL token, this will convert URLs pointing to a .gif,
* .jpg, or .png image into an inline <img /> tag (for the 'xhtml'
* format).
*
* Token options are:
*
* 'type' => ['inline'|'footnote'|'descr'] the type of URL
*
* 'href' => the URL link href portion
*
* 'text' => the displayed text of the URL link
*
* @category Text
*
* @package Text_Wiki
*
* @author Paul M. Jones <pmjones@php.net>
*
*/

class Text_Wiki_Parse_Url extends Text_Wiki_Parse {

    /**
    *
    * Keeps a running count of numbered-reference URLs.
    *
    * @access public
    *
    * @var int
    *
    */

    public $footnoteCount = 0;

    /**
    *
    * URL schemes recognized by this rule.
    *
    * @access public
    *
    * @var array
    *
    */

    public $conf = array(
        'schemes' => array(
            'http://',
            'https://',
            'ftp://',
            'gopher://',
            'news://',
            'mailto:',
            'mms://'
        )
    );

    /**
    *
    * Constructor.
    *
    * We override the constructor so we can comment the regex nicely.
    *
    * @access public
    *
    */

    function Text_Wiki_Parse_Url(&$obj)
    {
        parent::Text_Wiki_Parse($obj);

        // convert the list of recognized schemes to a regex-safe string,
        // where the pattern delim is a slash
        $tmp = array();
        $list = $this->getConf('schemes', array());
        foreach ($list as $val) {
            $tmp[] = preg_quote($val, '/');
        }
        $schemes = implode('|', $tmp);

        // build the regex
        $this->regex =
            "(?:(?:$schemes)" . // allowed schemes
            "(?:" . // start pattern
            "[^ \\/\"\'{$this->wiki->delim}]*\\/" . // no spaces, backslashes, slashes, double-quotes, single quotes, or delimiters;
            ")*" . // end pattern
            "[^ \\t\\n\\/\"\'{$this->wiki->delim}]*" .
            "[A-Za-z%0-9\\/?=&~_])";
        $this->regexLiberal =
            "(?:(?:$schemes)" . // allowed schemes
            "(?:" . // start pattern
            "[^ \\/\"\'{$this->wiki->delim}]*\\/" . // no spaces, backslashes, slashes, double-quotes, single quotes, or delimiters;
            ")*" . // end pattern
            "[^ \\t\\n\\/\"{$this->wiki->delim}]*)";

    }

    /**
    *
    * Find three different kinds of URLs in the source text.
    *
    * @access public
    *
    */

    function parse()
    {
        // -------------------------------------------------------------
        //
        // Described-reference (named) URLs.
        //

        // the regular expression for this kind of URL
        $tmp_regex =    "/" . 
                        "\[(\*)?" . 
                        "(" . 
                        "(?:".$this->regexLiberal.")" . 
                        "|" .
                        "(?:#[a-zA-Z0-9_\-%]*)" . 
                        "|" . 
                        "(?:\/[^\s\t\n\"'".$this->wiki->delim."]*)" . 
                        ")\s" . 
                        "([^\]".$this->wiki->delim."]+)\]" . 
                        "/x";

        // use a custom callback processing method to generate
        // the replacement text for matches.
        $this->wiki->source = preg_replace_callback(
            $tmp_regex,
            array(&$this, 'processDescr'),
            $this->wiki->source
        );

        $postVars = $this->getConf("post_vars");

        if($postVars){
            // enable %%foo%% variables to act in described links
            # What is this? I haven't seen it before
            $tmp_regex =    '/' . 
                            '\[' .             # Opening bracket
                            '(\*)?' . 
                            '(' . 
                            '%%' .             # Opening %%
                            '[^%]+' .          # Then at least one character not %
                            '%%' .             # Closing %%
                            '[^\s]*' .         # Then anything except whitespace
                            ')' . 
                            '\s' .             # A whitespace
                            '([^\]]+)' .       # Match anything up until a closing bracket
                            '\]' . 
                            '/x';
            $this->wiki->source = preg_replace_callback(
                $tmp_regex,
                array(&$this, 'processPV'),
                $this->wiki->source
	        );
        }

        // -------------------------------------------------------------
        //
        // Normal inline URLs.
        //

        // the regular expression for this kind of URL

        $tmp_regex =    '/' . 
                        '(^|[^A-Za-z])' .              # Start of line OR not letters
                        '(\*)?' .                      # Any amount of asterisks
                        '(' . $this->regex . ')' . 
                        '(.*?)' .                      # Then anything, but as few as possible?
                        '/x';

        // use the standard callback for inline URLs
        $this->wiki->source = preg_replace_callback(
            $tmp_regex,
            array(&$this, 'process'),
            $this->wiki->source
        );
    }

    /**
    *
    * Process inline URLs.
    *
    * @param array &$matches
    *
    * @param array $matches An array of matches from the parse() method
    * as generated by preg_replace_callback.  $matches[0] is the full
    * matched string, $matches[1] is the first matched pattern,
    * $matches[2] is the second matched pattern, and so on.
    *
    * @return string The processed text replacement.
    *
    */

    function process(&$matches)
    {
        // set options
        $options = array(
            'type' => 'inline',
            'href' => $matches[3],
            'text' => $matches[3]
        );

        if($matches[2] == '*'){
        		$options['target']='_blank';
        }
        // tokenize
        return $matches[1] . $this->wiki->addToken($this->rule, $options) . $matches[6];
    }

    /**
    *
    * Process described-reference (named-reference) URLs.
    *
    * Token options are:
    *     'type' => ['inline'|'footnote'|'descr'] the type of URL
    *     'href' => the URL link href portion
    *     'text' => the displayed text of the URL link
    *
    * @param array &$matches
    *
    * @param array $matches An array of matches from the parse() method
    * as generated by preg_replace_callback.  $matches[0] is the full
    * matched string, $matches[1] is the first matched pattern,
    * $matches[2] is the second matched pattern, and so on.
    *
    * @return string The processed text replacement.
    *
    */

    function processDescr(&$matches)
    {
        // set options
        $options = array(
            'type' => 'descr',
            'href' => $matches[2],
            'text' => $matches[3]
        );

        if($matches[1] == '*'){
        		$options['target']='_blank';
        }

        // tokenize
        return $this->wiki->addToken($this->rule, $options);
    }

     function processPV(&$matches)
    {
        // set options
        $options = array(
            'type' => 'descr',
            'href' => $matches[2],
            'text' => $matches[3]
        );

        if($matches[1] == '*'){
        		$options['target']='_blank';
        }

        // tokenize
        return $this->wiki->addToken($this->rule, $options);
    }
}
