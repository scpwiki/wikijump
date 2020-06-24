<?php
/**
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 * @license LGPL
 *
 * @version $Id$
 *
 */

/**
 *
 * Creates an iframe.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */
class Text_Wiki_Parse_Iframe extends Text_Wiki_Parse {

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

	public $regex = '';

	function __construct($obj){
		parent::__construct($obj);

        // convert the list of recognized schemes to a regex-safe string,
        // where the pattern delim is a slash
        $tmp = array();
        $list = $this->getConf('schemes', array());
        foreach ($list as $val) {
            $tmp[] = preg_quote($val, '/');
        }
        $schemes = implode('|', $tmp);

        // build the regex
        $urlRegex =
                    "(?:(?:$schemes)" .                         // allowed schemes
                    "(?:" .                                     // start pattern
                    "[^ \\/\"\'{$this->wiki->delim}]*\\/" .     // no spaces, backslashes, slashes, double-quotes, single quotes, or delimiters;
                    ")*" .                                      // end pattern
                    "[^ \\t\\n\\/\"{$this->wiki->delim}]*?)";

		$this->regex = '/\[\[iframe\s+('.$urlRegex.')(\s+.*?)?\]\]/si';

	}

    /**
    *
    * Generates a token entry for the matched text.  Token options are:
    *
    * 'text' => The full matched text, not including the <code></code> tags.
    *
    * @access public
    *
    * @param array &$matches The array of matches from parse().
    *
    * @return A delimited token number to be used as a placeholder in
    * the source text.
    *
    */

    function process(&$matches)
    {

    	$options = array();
    	$options['src'] = $matches[1];

    	$attr = $this->getAttrs(trim($matches[2]));

    	$iframeAttributes = array('align', 'frameborder', 'height', 'scrolling', 'width', 'class', 'style');

    	foreach($iframeAttributes as $a){
    		$options[$a] = $attr[$a];
    	}

    	$token = $this->wiki->addToken(
            $this->rule, $options
        );

        return $token;

    }

}
