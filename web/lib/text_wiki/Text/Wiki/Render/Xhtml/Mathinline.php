<?php
/**
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    $Id$
 * @link       http://pear.php.net/package/Text_Wiki
 */

/**
 * Inline math.
 *
 * @category   Text
 * @package    Text_Wiki
 * @author     Michal Frackowiak
 * @license    http://www.gnu.org/copyleft/lesser.html  LGPL License 2.1
 * @version    Release: @package_version@
 * @link       http://pear.php.net/package/Text_Wiki
 */
class Text_Wiki_Render_Xhtml_Mathinline extends Text_Wiki_Render {

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
        $content = $options['content'];
        $hashcode = md5($content);
        $runData = Ozone::getRunData();
        $site = $runData->getTemp('site');
        $dir = $site->getLocalFilesPath() . '/math/inline';
        if (!file_exists($dir)) {
            mkdirfull($dir);
        }

        $tmpDir = WIKIDOT_ROOT . '/tmp/math';
        if (!file_exists($tmpDir)) {
            mkdirfull($tmpDir);
        }

        $imgFile = $hashcode . '.png';
        if (!file_exists($dir . '/' . $imgFile)) {
            $renderer = new LatexRenderer();
            $renderer->setTmpDir($tmpDir);
            $renderer->setOutputDir($dir);
            $renderer->setDensity(110);
            $content2 = '$' . $content . '$';
            $renderer->render($content2, $hashcode);
        }
        if (!file_exists($dir . '/' . $imgFile)) {
            return '<span class="error-inline">' . _('The equation has not been processed correctly. Most prabably it has syntax error(s).') . '	</span>';
        }

        $out = '<img class="math-inline" src="/local--math/inline/' . $imgFile . '" alt="' . htmlentities($content) . '" />';

        return $out;

        $content = $options['content'];
        $hashcode = md5($content);
        $runData = Ozone::getRunData();
        $site = $runData->getTemp('site');
        $dir = $site->getLocalFilesPath() . '/math/inline';
        if (!file_exists($dir)) {
            mkdirfull($dir);
        }

        $imgFile = $hashcode . '.png';

        $imgFile = $hashcode . '.png';
        $out = '<img src="http://' . $site->getDomain() . '/local--math/inline/' . $imgFile . '" alt="' . htmlentities($content) . '" />';

        return $out;

    }
}
