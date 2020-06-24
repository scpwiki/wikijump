<?php

/**
*
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
* Parses for text marked as a math block.
*
* This class implements a Text_Wiki_Parse to find math invocation code,
* i.e. [[math label]] equation [[/math]]
*
* @category Text
*
* @package Text_Wiki
*
* @author Michal Frackowiak
*
*/

class Text_Wiki_Parse_Math extends Text_Wiki_Parse {

    static $equationsArray = array();

    /**
    *
    * The regular expression used to find source text matching this
    * rule.
    *
    * @access public
    *
    * @var string
    *
    */

    public $regex =     '/' . 
                        '^' . 
                        '\[\[math' .                # Start opening tag
                        '(\s+[a-z0-9_]*?)?' .       # Label
                        '((?:\s+' . 
                            '[a-z0-9]+="[^"]*"' .   # Parameters
                        '))*' .                     # Allow any number of parameters
                        '\s*\]\]' .                 # End opening tag
                        '((?:(?R)|.)*?)\n' .        # Contents - nesting is ok for some reason
                        '\[\[\/math\]\]' .          # Closing tag
                        '(\s|$)' . 
                        '/msix';

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

    function process(&$matches){

        if($this->wiki->vars['math_id'] == null){
    			$this->wiki->vars['math_id'] = 1;
    	}
    	$id = $this->wiki->vars['math_id'];
    	$this->wiki->vars['math_id']++;

	    $label = trim($matches[1]);
	    $content = trim($matches[3]);

	    if(preg_match('/\include\s*\{|\input\s*\{/is', $content)){
	    	throw new ProcessException("Invalid LaTeX expression(s) found.");
	    }

	    $args = $this->getAttrs($matches[2]);
	   	$type = $args['type'];
	   	if(!$type || !in_array($type, array('equation', 'eqnarray'))){
	   		$type = 'equation';
	   	}

        $options = array('label'=>$label, 'content'=>$content, 'id'=>$id, 'type' => $type);
        self::$equationsArray[$label] = $id;
        $id++;
        return "\n\n".$this->wiki->addToken($this->rule, $options)."\n\n";
    }
}
