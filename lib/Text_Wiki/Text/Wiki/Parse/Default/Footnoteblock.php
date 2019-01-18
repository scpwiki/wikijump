<?php

/**
* 
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
* Defines the place where footnotes whould be placed.
*
* @category Text
* 
* @package Text_Wiki
* 
* @author Michal Frackowiak
* 
*/

class Text_Wiki_Parse_Footnoteblock extends Text_Wiki_Parse {

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

	public $regex = null;
    
    function parse(){
    		$regex = '/(\[\[footnoteblock(\s+[^\]]+)?\]\])|($)/s';
    		$this->wiki->source = preg_replace_callback($regex, array($this,"process"),	$this->wiki->source, 1);
    }
    
    function process($matches)
    {
    		$footnotes = $this->wiki->vars['footnotes'];
    		if(count($footnotes) == 0){return '';} // render nothing if no footnotes.
    		$inside = '';
    		$fni = $this->wiki->parseObj['Footnoteitem'];

    		foreach($footnotes as $id => $content){
    			$inside.=$this->wiki->addToken($fni->rule, array('type'=>'start', 'id' => $id+1))
    					.$content."\n\n"
    					.$this->wiki->addToken($fni->rule, array('type'=>'end', 'id' => $id+1));
    		}
    	
    	$args = $this->getAttrs($matches[2]);
    	$title = $args['title'];
        return "\n".$this->wiki->addToken($this->rule, array('type'=>'start', 'title' => $title))
        		.$inside
        		.$this->wiki->addToken($this->rule, array('type'=>'end'))."\n";
    }
}
