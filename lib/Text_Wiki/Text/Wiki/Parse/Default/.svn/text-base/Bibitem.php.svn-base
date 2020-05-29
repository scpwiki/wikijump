<?php

/**
 * 
 * Parses bibliography items within the [[bibliography]] block.
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
 * Parses bibliography items within the [[bibliography]] block.
 *
 * @category Text
 * 
 * @package Text_Wiki
 * 
 * @author Michal Frackowiak
 * 
 */

class Text_Wiki_Parse_Bibitem extends Text_Wiki_Parse {

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

    function process(&$matches) {
        $label = $matches[1];
        $content = $matches[2];
        $content = trim(str_replace("\n", " ", $content));
        $this->wiki->vars['bibitems'][$label] = $content;
        $id = count($this->wiki->vars['bibitemIds']) + 1;
        $this->wiki->vars['bibitemIds'][$label] = $id;
        return $this->wiki->addToken($this->rule, 
                array(
                    'label' => $label, 
                    'id' => $id, 
                    'type' => 'start')) .
                 $content . $this->wiki->addToken(
                        $this->rule, 
                        array(
                            'type' => 'end'));
    }
}
