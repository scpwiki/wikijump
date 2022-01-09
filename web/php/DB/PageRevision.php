<?php
declare(strict_types=1);

namespace Wikidot\DB;

use Illuminate\Support\Facades\Cache;
use Exception;
use Wikijump\Models\User;
use Wikijump\Services\Deepwell\DeepwellService;

/**
 * Object Model Class.
 *
 */
class PageRevision extends PageRevisionBase
{
    public function getSourceText(): string
    {
        return DeepwellService::getInstance()->getText($this->getWikitextHash());
    }

    public function resetFlags(): void
    {
        $this->setFlagText(false);
        $this->setFlagTitle(false);
        $this->setFlagRename(false);
        $this->setFlagNew(false);
        $this->setFlagFile(false);
    }

    public function getUser(): ?User
    {
        if ($this->getUserId() == User::ANONYMOUS_USER) {
            return null;
        }

        return User::find($this->getUserId());
    }

    /**
    * @return User|string
    */
    public function getUserOrString()
    {
        $user = $this->getUser();
        if ($user == null) {
            return $this->getUserString();
        } else {
            return $user;
        }
    }

    public function getMetadata(): PageMetadata
    {
        return PageMetadataPeer::instance()->selectByPrimaryKey($this->getMetadataId());
    }

    public function getPage(): ?Page
    {
        if (is_array($this->prefetched)) {
            if (in_array('page', $this->prefetched)) {
                if (in_array('page', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['page'];
                } else {
                    $obj = new Page($this->sourceRow);
                    $obj->setNew(false);
                    $this->prefetchedObjects['page'] = $obj;
                    return $obj;
                }
            }
        }
        return PagePeer::instance()->selectByPrimaryKey($this->getPageId());
    }

    public function save(): void
    {
        try {
            $page = $this->getPage();
            if ($page) {
                $key = "sitechangesfeed..".$page->getSiteId();
                Cache::forget($key);

                $tkey = "siterevisions_lc..".$page->getSiteId();
                Cache::put($tkey, time(), 3600);

                $tkey = "pagerevisions_lc..".$page->getPageId();
                Cache::put($tkey, time(), 3600);
            }
        } catch (Exception $e) {
        }

        parent::save();
    }
}
