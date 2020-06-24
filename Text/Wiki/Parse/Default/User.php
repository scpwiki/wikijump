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
 * Parses for user info.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */

class Text_Wiki_Parse_User extends Text_Wiki_Parse {

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
                        '\[\[' . 
                        '(\*)?' .      # Optional asterisk
                        'user\s' . 
                        '([^\]]+)' .   # Parameters (e.g. user name)
                        '\]\]' . 
                        '/ix';

    function process(&$matches) {
        $userName = $matches[2];
        $options = array('userName' => $userName);
        if ($matches[1] == '*') {
            $options['image'] = true;
        }

        return $this->wiki->addToken($this->rule, $options);

    }
}
