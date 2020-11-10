<?php
/**
 *
 * Creates a button, e.g. edit, history etc.
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
 * Creates a button, e.g. edit, history etc.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */
class Text_Wiki_Parse_Button extends Text_Wiki_Parse {

    public $regex =     '/' . 
                        '\[\[' .             # Opening brackets
                        'button\s+' .        # Tag name
                        '([a-z0-9\-_]+)' .   # Button name
                        '(?:\s+(.+?))?' .    # Optional button parameters
                        '\]\]' .             # Closing brackets
                        '/isx';

    function process(&$matches)
    {
        $type = $matches[1];
        $attrString = $matches[2];
        $attr = $this->getAttrs(trim($attrString));

        $allowedAttrs = array('text', 'class', 'style');
        $options = array();
        foreach($allowedAttrs as $aa){
        	if(isset($attr[$aa])){
        		$options[$aa] = $attr[$aa];
        	}
        }

        $type = str_replace("_", "-", $type);

        $options['type'] = $type;

        return $this->wiki->addToken($this->rule, $options);

    }
}
