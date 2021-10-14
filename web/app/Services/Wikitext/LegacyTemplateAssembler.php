<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ForumThreadPeer;
use Wikidot\DB\PagePeer;
use Wikidot\Utils\GlobalProperties;
use Wikijump\Helpers\LegacyTools;
use Wikijump\Models\User;

/**
 * Legacy class to assemble _template files.
 * This just copies the logic originally held in WikiTransformation.
 * It's a big mess. TODO clean this nonsense up.
 * @package Wikijump\Services\Wikitext
 */
// prettier-ignore
final class LegacyTemplateAssembler
{
    private function __construct() {}

    public static function assembleTemplate($source, $template, $page = null)
    {
        /* First check if it is a real "live" template. If not, return the original $source.
         * To be recognized as a live template it mast contain either %%content%% or
         * %%content{X}%% tags. */

        /* Handle ListPages module inside a template -- %%content%% need to be escaped. */
        $template = preg_replace_callback(
            ";^\\[\\[module\\s+ListPages(.*?)\n\\[\\[/module\\]\\];ms",
            fn(array $matches) => self::handleListPages($matches),
            $template,
        );
        $template = preg_replace_callback(
            ";^\\[\\[module\\s+NextPage(.*?)\n\\[\\[/module\\]\\];ms",
            fn(array $matches) => self::handleListPages($matches),
            $template
        );
        $template = preg_replace_callback(
            ";^\\[\\[module\\s+PreviousPage(.*?)\n\\[\\[/module\\]\\];ms",
            fn(array $matches) => self::handleListPages($matches),
            $template,
        );
        $template = preg_replace_callback(
            ";^\\[\\[module\\s+Feed(.*?)\n\\[\\[/module\\]\\];ms",
            fn(array $matches) => self::handleListPages($matches),
            $template,
        );
        $template = preg_replace_callback(
            ";^\\[\\[module\\s+FrontForum(.*?)\n\\[\\[/module\\]\\];ms",
            fn(array $matches) => self::handleListPages($matches),
            $template,
        );

        if (!preg_match(';%%content({[0-9]+})?%%;', $template)) {
            return $source;
        }
        $out = $source;

        $template = preg_replace(';%%content({[0-9]+})?%%;', '%%%\\0%%%', $template);
        $template = preg_replace(';(?<!%)%%[a-z0-9\(\)_]+%%(?!%);i', '%%%\\0%%%', $template);
        $template = preg_replace(';(?<!%)%%date(\|.*?)?%%(?!%);i', '%%%\\0%%%', $template);

        $template = preg_replace(";%\xFA%(content({[0-9]+}))?%\xFA%;", "%%\\1%%", $template);
        $template = preg_replace(";%\xFA%([a-z0-9\(\)_]+)%\xFA%;i", '%%\\1%%', $template);
        $template = preg_replace(";%\xFA%(date(\|.*?)?)%\xFA%;i", '%%\\1%%', $template);

        /* Check if has a ===== delimiter. */
        $split = preg_split(';^={4,}$;sm', $template);
        if (count($split) > 1) {
            $template = trim($split[0]);
        }

        /* If there is $page, try substituting more tags. */
        if ($page) {
            $b = $template;
            $title = $page->getTitle();
            $title = str_replace(array('[', ']'), '', $title);
            $replacement = preg_quote_replacement('[[[' . $page->getUnixName() . ' | ' . $title . ']]]');
            $b = str_replace('%%%%%title%%%%%', $title, $b);
            $b = preg_replace(';%%%%%((linked_title)|(title_linked))%%%%%;i', $replacement, $b);

            if ($page->getOwnerUserId()) {
                $user = User::find($page->getOwnerUserId());
                if (LegacyTools::isSystemAccount($user->id) === false) {
                    $userString = '[[*user ' . $user->username . ']]';
                } else {
                    $userString = _('Anonymous user');
                }
            } else {
                $userString = _('Anonymous user');
            }
            $b = str_ireplace('%%%%%author%%%%%', $userString, $b);
            $b = str_ireplace('%%%%%user%%%%%', $userString, $b);

            $b = str_ireplace('%%%%%user_edited%%%%%', $userString, $b);

            $b = preg_replace(
                ';%%%%%date(\|.*?)?%%%%%;',
                '%%%%%date|' . $page->getDateCreated()->getTimestamp() . '\\1%%%%%',
                $b,
            );
            $b = preg_replace(
                ';%%%%%date_edited(\|.*?)?%%%%%;',
                '%%%%%date|' . $page->getDateLastEdited()->getTimestamp() . '\\1%%%%%',
                $b,
            );

            /* %%rating%% */
            $b = str_ireplace('%%%%%rating%%%%%', $page->getRate(), $b);

            /* %%comments%% */
            $b = preg_replace_callback(
                '/%%%%%comments%%%%%/i',
                fn(array $matches) => self::handleCommentsCount($page),
                $b,
            );

            /* %%page_unix_name%% */
            $b = str_ireplace('%%%%%page_unix_name%%%%%', $page->getUnixName(), $b);

            if (strpos($page->getUnixName(), ':') != false) {
                $tmp0 = explode(':', $page->getUnixName());
                $categoryName00 = $tmp0[0];
            } else {
                $categoryName00 = '_default';
            }

            $b = str_ireplace('%%%%%category%%%%%', $categoryName00, $b);

            /* %%link%% */
            $site = $page->getSite();
            $b = str_ireplace(
                '%%%%%link%%%%%',
                GlobalProperties::$HTTP_SCHEMA . '://' . $site->getDomain() . '/' . $page->getUnixName(),
                $b,
            );

            /* %%tags%% */
            $b = preg_replace_callback(
                '/%%%%%tags%%%%%/i',
                fn(array $matches) => self::handleTags($matches, $page),
                $b,
            );

            $b = preg_replace_callback(
                ';%%%%%date\|([0-9]+)(\|.*?)?%%%%%;',
                fn(array $matches) => self::formatDate($matches),
                $b,
            );

            $template = $b;
        }

        $out = str_replace('%%%%%content%%%%%', trim($out), $template);

        /* Handle split sources. */
        $splitSource = preg_split('/^([=]{4,})$/m', $source);
        for ($i = 0; $i < count($splitSource); $i++) {
            $out = str_replace('%%%%%content{' . ($i + 1) . '}%%%%%', trim($splitSource[$i]), $out);
        }
        return preg_replace(';%%%%%content({[0-9]+})?%%%%%;', '', $out);
    }

    private static function handleListPages($m)
    {
        if (preg_match(';^\[\[module;sm', $m[1])) {
            return $m[0];
        } else {
            $b = preg_replace(';%%(content({[0-9]+}))?%%;', "%\xFA%\\1%\xFA%", $m[0]);
            $b = preg_replace(';(?<!%)%%([a-z0-9\(\)_]+)%%(?!%);i', "%\xFA%\\1%\xFA%", $b);
            $b = preg_replace(';(?<!%)%%(date(\|.*?)?)%%(?!%);i', "%\xFA%\\1%\xFA%", $b);
            return $b;
        }
    }

    private static function handleCommentsCount($page)
    {
        $threadId = $page->getThreadId();
        if ($threadId) {
            $thread = ForumThreadPeer::instance()->selectByPrimaryKey($threadId);
            if ($thread) {
                return $thread->getNumberPosts();
            }
        }
        return 0;
    }

    private static function formatDate($m)
    {
        if (isset($m[2])) {
            $format = preg_replace(';^\|;', '', $m[2]);
        } else {
            $format = '%e %b %Y, %H:%M %Z|agohover';
        }
        return '[[date ' . $m[1] . ' format="' . $format . '"' . ']]';
    }

    private static function handleTags($m, $page)
    {
        /* Select tags. */
        // get the tags
        $t2 = PagePeer::getTags($pageId);
        if (count($t2) == 0) {
            return _('//no tags found for this page//');
        }
        return implode(' ', $t2);
    }
}
