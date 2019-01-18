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
 * Parses for typographic elements.
 *
 * @category Text
 * 
 * @package Text_Wiki
 * 
 * @author Michal Frackowiak
 * 
 */

class Text_Wiki_Parse_Typography extends Text_Wiki_Parse {

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
    
    function process(&$matches) {

    }

    public function parse() {
        $source = $this->wiki->source;
        
        // fix a few typography things now...
        

        //quotes
        $source = preg_replace("/``(.*?)''/", $this->wiki->addToken($this->rule, array(
            'char' => '``')) . '$1' . $this->wiki->addToken($this->rule, array(
            'char' => "''")), $source);
        $source = preg_replace("/,,(.*?)''/", $this->wiki->addToken($this->rule, array(
            'char' => ',,')) . '$1' . $this->wiki->addToken($this->rule, array(
            'char' => "''")), $source);
        
        $source = preg_replace("/`(.*?)'/", $this->wiki->addToken($this->rule, array(
            'char' => '`')) . '$1' . $this->wiki->addToken($this->rule, array(
            'char' => "'")), $source);
        
        // french
        $source = preg_replace("/<</", $this->wiki->addToken($this->rule, array(
            'char' => '<<')), $source);
        $source = preg_replace("/>>/", $this->wiki->addToken($this->rule, array(
            'char' => '>>')), $source);
        
        // numbers and units
        $source = preg_replace("/(?<=[0-9]) (?=[0-9])/", $this->wiki->addToken($this->rule, array(
            'char' => ' ')), $source);
        
        $units = '
		### Metric units (with prefixes)
		(?:
			p |
			µ | &micro; | &\#0*181; | &\#[xX]0*[Bb]5; |
			[mcdhkMGT]
		)?
		(?:
			[mgstAKNJWCVFSTHBL]|mol|cd|rad|Hz|Pa|Wb|lm|lx|Bq|Gy|Sv|kat|
			Ω | Ohm | &Omega; | &\#0*937; | &\#[xX]0*3[Aa]9;
		)|
		### Computers units (KB, Kb, TB, Kbps)
		[kKMGT]?(?:[oBb]|[oBb]ps|flops)|
		### Money
		¢ | &cent; | &\#0*162; | &\#[xX]0*[Aa]2; |
		M?(?:
			£ | &pound; | &\#0*163; | &\#[xX]0*[Aa]3; |
			¥ | &yen;   | &\#0*165; | &\#[xX]0*[Aa]5; |
			€ | &euro;  | &\#0*8364; | &\#[xX]0*20[Aa][Cc]; |
			$
		)|
		### Other units
		(?: ° | &deg; | &\#0*176; | &\#[xX]0*[Bb]0; ) [CF]? | 
		%|pt|pi|M?px|em|en|gal|lb|[NSEOW]|[NS][EOW]|ha|mbar
		'; //x
        

        $source = preg_replace('/
			(?:([0-9])[ ]) # Number followed by space.
			(' . $units . ')     # Unit.
			(?![a-zA-Z0-9])  # Negative lookahead for other unit characters.
			/x', "\\1" . $this->wiki->addToken($this->rule, array('char' => ' ')) . "\\2", $source);
        
        $source = str_replace(array("...", ". . ."), $this->wiki->addToken($this->rule, array(
            'char' => '...')), $source);
        
        // dashes
        $source = preg_replace('/\-\-/', $this->wiki->addToken($this->rule, array(
            'char' => '--')), $source);
        $source = preg_replace('/\-\-\-/', $this->wiki->addToken($this->rule, array(
            'char' => '---')), $source);
        
        $this->wiki->source = $source;
    }
}
