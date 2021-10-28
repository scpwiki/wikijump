<?php
declare(strict_types=1);

namespace Wikijump\Models;

use Illuminate\Contracts\Validation\Validator;
use Illuminate\Database\Eloquent\Model;

class PageLink extends Model
{
    /**
     * Manually setting the name of the table.
     *
     * @var string
     */
    protected $table = 'page_link';

    /**
     * The attributes that are mass assignable.
     *
     * @var array
     */
    protected $fillable = ['page_id', 'site_id', 'url', 'count'];

    /**
     * Creates a validator for this database object.
     *
     * @param array $data
     * @return Validator
     */
    protected function validator(array $data)
    {
        return Validator::make($data, [
            'count' => 'min:1',
        ]);
    }
}
