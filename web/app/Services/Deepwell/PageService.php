<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\PageRevision;
use Wikidot\DB\PageRevisionPeer;

final class PageService
{
    /**
     * @param string|int $page_id
     * @return PageRevision
     */
    public static function getLatestRevision($page_id): PageRevision
    {
        $c = new Criteria();
        $c->add('page_id', (string) $page_id);
        $c->addOrderDescending('revision_id');
        return PageRevisionPeer::instance()->selectOne($c);
    }

    /**
     * @param string|int $page_id
     * @param array $fields List of fields to get
     * @return array Requested fields and the revision
     */
    public static function getLatestContents($page_id, array $fields): array
    {
        $deepwell = DeepwellService::getInstance();
        $revision = self::getLatestRevision($page_id);
        $result = ['revision' => $revision];

        if (in_array('wikitext', $fields)) {
            $result['wikitext'] = $deepwell->getText($revision->getWikitextHash());
        }

        if (in_array('compiled_html', $fields)) {
            $result['compiled_html'] = $deepwell->getText($revision->getCompiledHash());
        }

        if (in_array('compiled_generator', $fields)) {
            $result['compiled_generator'] = $revision->getCompiledGenerator();
        }

        return $result;
    }
}
