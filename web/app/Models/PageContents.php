<?php
declare(strict_types=1);

namespace Wikijump\Models;

use Illuminate\Database\Eloquent\Model;
use Ozone\Framework\Database\Criteria;
use Wikidot\DB\PageRevision;
use Wikidot\DB\PageRevisionPeer;

class PageContents extends Model
{
    /**
     * The attributes that are mass assignable.
     *
     * @var array
     */
    protected $fillable = [
        'revision_id',
        'wikitext',
        'compiled_html',
        'generator',
    ];

    private static function getLatestRevision(string $page_id): PageRevision
    {
        $c = new Criteria();
        $c->add('page_id', $page_id);
        $c->addOrderDescending('revision_id');
        return PageRevisionPeer::instance()->selectOne($c);
    }

    private static function getLatest(string $page_id, array $columns): PageContents
    {
        $revision_id = self::getLatestRevision($page_id)->getRevisionId();
        return PageContents::where('revision_id', $revision_id)
            ->select($columns)
            ->first();
    }

    public static function getLatestFull(string $page_id): PageContents
    {
        return self::getLatest($page_id, ['revision_id', 'wikitext', 'compiled_html', 'generator']);
    }

    public static function getLatestWikitext(string $page_id): PageContents
    {
        return self::getLatest($page_id, ['wikitext']);
    }

    public static function getLatestCompiledHtml(string $page_id): PageContents
    {
        return self::getLatest($page_id, ['compiled_html', 'generator']);
    }
}
