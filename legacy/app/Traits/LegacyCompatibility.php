<?php
declare(strict_types=1);

namespace Wikijump\Traits;

use Illuminate\Support\Facades\Schema;
use Ozone\Framework\Database\BaseDBPeer;

trait LegacyCompatibility
{
    /**
     * Legacy models that are used in joins get this method from the BaseDBPeer.
     * It returns all the columns in a table and renames them to not clash.
     * @return string
     * @see BaseDBPeer
     */
    public function fieldListStringSpecialJoin(): string
    {
        $table = $this->getTable();
        $columns = Schema::getColumnListing($table);
        $modifiedColumns = [];
        foreach ($columns as $column) {
            $modifiedColumns[] = "$table.$column AS ${table}___$column";
        }

        return implode(', ', $modifiedColumns);
    }
}
