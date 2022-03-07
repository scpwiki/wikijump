<?php
/**
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    $Id$
 * @link       http://pear.php.net/package/Text_Wiki
 */

use Wikidot\Utils\ModuleManager;

/**
 * Module.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Module extends Text_Wiki_Render {

    public $conf = array();

    /**
     *
     * Renders a token into text matching the requested format.
     *
     * @access public
     *
     * @param array $options The "options" portion of the token (second
     * element).
     *
     * @return string The text rendered from the token options.
     *
     */

    function token($options) {
        $moduleName = $options['moduleName'];
        $siteName = $GLOBALS['site']->getSlug();
        // check if the site is allowed to use this module
	$mm = ModuleManager::instance();
        if ($mm->canWikiUseModule($siteName, $moduleName) == false) {
            return $this->renderError(sprintf(_("Module <em>%s</em> does not exist or cannot be used within this site."), $moduleName));
        }

        $attr = $options['attr'];
        if ($options['module_body']) {
            $attr .= ' module_body="' . urlencode($options['module_body']) . '"';
        }
        $templateName = $mm->resolveWikiModuleName($moduleName);
        $d = utf8_encode("\xFE");
        $out = $d . "module \"" . $templateName . "\"";
        if ($attr !== null && $attr !== '') {
            $out .= " " . urlencode($attr) . " ";
        }
        $out .= $d;

        return $out;
    }
}
