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
* Parses for [[size]] tags.
*
* @category Text
*
* @package Text_Wiki
*
* @author Michal Frackowiak
*
*/

class Text_Wiki_Parse_Size extends Text_Wiki_Parse {

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
                        '\[\[size\s([^\]]+)\]\]' .   # Opening tag including parameters
                        '((?:(?R)|.)*?)' .           # Any content in between including other sizes
                        '\[\[\/size\]\]' .           # Closing tag
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

    function process(&$matches)
    {

    		$content =$matches[2];

    		$size = trim($matches[1]);

    		$allowedSizes = array(
                '/^[0-9\.]{1,5}(em|px|%)$/',
                '/^xx\-small$/',
                '/^x\-small$/',
                '/^small$/',
                '/^medium$/',
                '/^large$/',
                '/^x\-large$/',
                '/^xx\-large$/',
                '/^smaller$/',
                '/^larger$/'
    		);

    		$good = false;
    		foreach($allowedSizes as $as){
    			if(preg_match($as, $size)){
    				$good=true;
    				break;
    			}
    		}

    		if($good==false){
    			return $matches[0];
    		}

        $options = array('size'=>$size, 'type'=>'start');

        $start = $this->wiki->addToken($this->rule, $options);

        $end = $this->wiki->addToken(
            $this->rule, array('type' => 'end')
        );

        return $start . $content  .$end;

    }

    function parse(){
    	$oldSource = $this->wiki->source;
        $this->wiki->source = preg_replace_callback(
           	$this->regex,
           	array(&$this, 'process'),
           	$this->wiki->source
        );
        if($oldSource != $this->wiki->source){
        	$this->parse();
        }
    }

}
