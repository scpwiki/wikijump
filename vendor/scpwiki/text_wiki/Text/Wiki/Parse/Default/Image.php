<?php

/**
 *
 * Parses for image placement.
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
 * Parses for image placement.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Paul M. Jones <pmjones@php.net>
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_Image extends Text_Wiki_Parse {

    /**
     * URL schemes recognized by this rule.
     *
     * @access public
     * @var array
     */
    public $conf = array('schemes' => 'http|https|ftp|gopher|news',
        'host_regexp' => '(?:[^.\s/"\'<\\\#delim#\ca-\cz]+\.)*[a-z](?:[-a-z0-9]*[a-z0-9])?\.?',
        'path_regexp' => '(?:/[^\s"<\\\#delim#\ca-\cz]*)?');

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
                        '(' . 
                        '\[\[' .                  # Start opening image tag
                        '((?:f)?[<>=])?' .        # Allow f for flickr integration and <, >, =
                        'image' . 
                        '\s+' .                   # Require a whitespace before parameters
                        ')' . 
                        '(.+?)' .                 # Parameters
                        '(?:\]\])' .              # End opening image tag
                        '(?:(.*?)' .              # Capture any text inside the image element
                        '\[\[\/image\]\])?' .     # Allow end tag. Content + end tag is not required
                        '/isx';

    /**
     * The regular expressions used to check ecternal urls
     *
     * @access public
     * @var string
     * @see parse()
     */
    public $url = '';

    /**
     * Constructor.
     * We override the constructor to build up the url regex from config
     *
     * @param object &$obj the base conversion handler
     * @return The parser object
     * @access public
     */
    function Text_Wiki_Parse_Image(&$obj) {
        $default = $this->conf;
        parent::Text_Wiki_Parse($obj);

        // convert the list of recognized schemes to a regex OR,
        $schemes = $this->getConf('schemes', $default['schemes']);
        $this->url = str_replace('#delim#', $this->wiki->delim, '#(?:' . (is_array($schemes) ? implode('|', $schemes) : $schemes) . ')://' . $this->getConf('host_regexp', $default['host_regexp']) . $this->getConf('path_regexp', $default['path_regexp']) . '#');
    }

    /**
     *
     * Generates a token entry for the matched text.  Token options are:
     *
     * 'src' => The image source, typically a relative path name.
     *
     * 'opts' => Any macro options following the source.
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
        $pos = strpos($matches[3], ' ');

        if ($pos === false) {
            $options = array('src' => $matches[3],
                'attr' => array());
        } else {
            // everything after the space is attribute arguments

            // from the list of arguments leave ONLY:
            // class, link and style
            $attr = $this->getAttrs(substr($matches[3], $pos + 1));
            $attr2 = array();
            if (isset($attr['link'])) {
                $attr2['link'] = $attr['link'];
            }
            if (isset($attr['style'])) {
                $attr2['style'] = $attr['style'];
            }
            if (isset($attr['class'])) {
                $attr2['class'] = $attr['class'];
            }
            if (isset($attr['alt'])) {
                $attr2['alt'] = $attr['alt'];
            }
            if (isset($attr['height'])) {
                $attr2['height'] = $attr['height'];
            }
            if (isset($attr['width'])) {
                $attr2['width'] = $attr['width'];
            }
            if (isset($attr['size'])) {
                $attr2['size'] = strtolower($attr['size']);
            }
            $options = array(
                'src' => substr($matches[3], 0, $pos),
                'attr' => $attr2);

            // fix the target="_blank"

            $link = $options['attr']['link'];
            if ($link && $link[0] == '*') {
                $link = substr($link, 1);
                $options['target'] = '_blank';
                $options['attr']['link'] = $link;
            }

            // check the scheme case of external link
            if (array_key_exists('link', $options['attr'])) {
                // external url ?
                if (($pos = strpos($options['attr']['link'], '://')) !== false) {
                    if (!preg_match($this->url, $options['attr']['link'])) {
                        return $matches[0];
                    }
                } elseif (in_array('Wikilink', $this->wiki->disable)) {
                    // return $matches[0]; // Wikilink disabled
                }
            }
        }

        $align = $matches[2];

        if ($align != null && $align !== '') {
            $options['align'] = $align;
        }

        $caption = $matches[4];
        if ($caption != null && $caption !== '') {
            $options['caption'] = $caption;
        }

        // fix the target="_blank"

        $src = $options['src'];
        if ($src && $src[0] == '*') {
            $src = substr($src, 1);
            $options['target'] = '_blank';
            $options['src'] = $src;
        }

        return ($align ? "\n\n" : '') . $this->wiki->addToken($this->rule, $options) . ($align ? "\n\n" : '');
    }
}
