<?php
/**
 * 
 * @category Text
 * 
 * @package Text_Wiki
 * 
 * @author Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 * 
 * @license LGPL
 * 
 * @version $Id$
 * 
 */

/**
 * 
 * Parses for advanced table.
 * 
 * @category Text
 * 
 * @package Text_Wiki
 * 
 * @author Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 * 
 */

class Text_Wiki_Parse_Tableadv extends Text_Wiki_Parse {
    
    public $regex = ";\n\[\[table(\s.*?)?\]\](\s*(?:\[\[row(?:\s[^\]]*)?\]\]\s*(?:\[\[(column|col|cell)(?:\s[^\]]*)?\]\](?:(?R)|.)*?\[\[/(column|col|cell)\]\]\s*)+\[\[/row\]\]\s*)+)\[\[/table\]\]\n;sxi";
    
    private $_tmpSource = null;

    function process(&$matches) {
        $content = $matches[2];
        $attr = $this->getAttrs(trim($matches[1]));
        $options = array();
        if ($attr['class']) {
            $options['class'] = $attr['class'];
        }
        if ($attr['style']) {
            $options['style'] = $attr['style'];
        }
        
        // look for nested tables - TODO
        

        $content = preg_replace_callback($this->regex, array(&$this, 
            'process'), $content);
        
        // look for rows
        $content = preg_replace_callback(';' . "(?:\n)?" . '\[\[row(\s[^\]]*)?\]\]\s*((\[\[(column|col|cell)(\s[^\]]*)?\]\].*?\[\[/(column|col|cell)\]\]\s*)+)\[\[/row\]\]' . "(?:\n)?" . ';msi', array(
            $this, '_handleRow'), $content);
        
        $start = $this->wiki->addToken($this->rule, array_merge($options, array(
            'type' => 'start')));
        $end = $this->wiki->addToken($this->rule, array(
            'type' => 'end'));
        return "\n\n" . $start . $content . $end . "\n\n";
    }

    function parse() {
        
        $this->wiki->source = preg_replace_callback($this->regex, array(
            &$this, 'process'), $this->wiki->source);
    }

    private function _handleRow($matches) {
        $content = $matches[2];
        $attr = $this->getAttrs(trim($matches[1]));
        
        $content = preg_replace_callback(';' . "(?:\n)?" . '\[\[(?:column|col|cell)(\s[^\]]*)?\]\](.*?)\[\[/(column|col|cell)\]\]' . "(?:\n)?" . ';msi', array(
            $this, '_handleCell'), $content);
        $options = array();
        if ($attr['class']) {
            $options['class'] = $attr['class'];
        }
        if ($attr['style']) {
            $options['style'] = $attr['style'];
        }
        $start = $this->wiki->addToken($this->rule, array_merge($options, array(
            'type' => 'rowStart')));
        $end = $this->wiki->addToken($this->rule, array(
            'type' => 'rowEnd'));
        return $start . $content . $end;
    
    }

    private function _handleCell($matches) {
        $content = $matches[2];
        $attr = $this->getAttrs(trim($matches[1]));
        $options = array();
        if ($attr['class']) {
            $options['class'] = $attr['class'];
        }
        if ($attr['style']) {
            $options['style'] = $attr['style'];
        }
        
        // newlines if no paragraps inside
        if (strpos($content, "\n\n") === false) {
            $content = trim($content);
        } else {
            $content = "\n\n" . $content . "\n\n";
        }
        
        $start = $this->wiki->addToken($this->rule, array_merge($options, array(
            'type' => 'cellStart')));
        $end = $this->wiki->addToken($this->rule, array(
            'type' => 'cellEnd'));
        return $start . $content . $end;
    }
}
