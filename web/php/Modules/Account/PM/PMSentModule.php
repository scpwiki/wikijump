<?php

declare(strict_types=1);

namespace Wikidot\Modules\Account\PM;

use Wikidot\Utils\AccountBaseModule;
use Wikijump\Models\UserMessage;

/**
 * AJAX Module for per-page Sent items view
 * @package Wikidot\Modules\Account\PM
 */
class PMSentModule extends AccountBaseModule
{

    /**
     * Build per-page sent messages view
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

        $sent_count = UserMessage::sent($runData->user())->count();

        /** Build the pager. */
        $total_pages  = ceil($sent_count / $per_page_limit);

        /** Do not allow a `page` value greater than the total. */
        $this_page = min($this_page, $total_pages);

        $offset = max(($this_page - 1) * $per_page_limit, 0);

        $messages = UserMessage::sent($runData->user())
            ->orderBy('id', 'DESC')
            ->limit($per_page_limit)
            ->offset($offset)
            ->get();

        $runData->contextAdd('totalPages', $total_pages);
        $runData->contextAdd('currentPage', $this_page);
        $runData->contextAdd('count', $sent_count);
        $runData->contextAdd('totalPages', $total_pages);
        $runData->contextAdd('messages', $messages);
    }
}
