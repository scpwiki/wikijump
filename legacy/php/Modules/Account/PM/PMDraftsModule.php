<?php

declare(strict_types=1);

namespace Wikidot\Modules\Account\PM;

use Wikidot\Utils\AccountBaseModule;
use Wikijump\Models\UserMessage;

/**
 * AJAX Module for per-page view of drafts.
 * @package Wikidot\Modules\Account\PM
 */
class PMDraftsModule extends AccountBaseModule
{

    /**
     * Build drafts view.
     * @param $runData
     */
    public function build($runData)
    {
        /** Not sure if this is worth breaking out into a setting. */
        $per_page_limit = 30;

        $this_page = $runData->get('page');
        if ($this_page === null || $this_page < 0) {
            $this_page = 1;
        }

        $drafts_count = UserMessage::drafts($runData->user())->count();

        /** Build the pager. */
        $total_pages  = ceil($drafts_count / $per_page_limit);

        /** Do not allow a `page` value greater than the total. */
        $this_page = min($this_page, $total_pages);

        $offset = max(($this_page - 1) * $per_page_limit, 0);

        $messages = UserMessage::drafts($runData->user())
            ->orderBy('id', 'DESC')
            ->limit($per_page_limit)
            ->offset($offset)
            ->get();

        $runData->contextAdd('totalPages', $total_pages);
        $runData->contextAdd('currentPage', $this_page);
        $runData->contextAdd('count', $drafts_count);
        $runData->contextAdd('totalPages', $total_pages);
        $runData->contextAdd('messages', $messages);
    }
}
