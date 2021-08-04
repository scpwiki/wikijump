<?php
declare(strict_types=1);

namespace Wikijump\Traits;

use Illuminate\Database\Eloquent\Collection;
use Wikijump\Models\Interaction;

trait HasInteractions {

    /**
     * Retrieves a collection of objects related to the caller via an Interaction.
     * @param int $interaction_type
     * @return Collection
     */
    public function my(int $interaction_type) : Collection
    {
        /**
         * Pull a collection of Interactions given an Interaction type where
         * the calling object is the setter.
         */
        $list = $this->morphMany(Interaction::class, 'setter')
            ->where('interaction_type', $interaction_type)
            ->get();
        if($list->count() === 0) { return $list; }
        else
        {
            # Retrieve the class type of the target.
            $class = $list->first()->pluck('target_type')[0];
            # Get the list of IDs.
            $ids = $list->pluck('target_id')->toArray();
            # Use the whereIn method of the class to return a collection of class objects.
            return $class::whereIn('id', $ids)->get();
        }
    }

    /**
     * Retrieves a collection of objects related to the target via an Interaction.
     * @param int $interaction_type
     * @return Collection
     */
    public function their(int $interaction_type) : Collection
    {
        /**
         * Pull a collection of Interactions given an Interaction type where
         * the calling object is the target.
         */
        $list = $this->morphMany(Interaction::class, 'target')
            ->where('interaction_type', $interaction_type)
            ->get();
        if ($list->count() === 0)
        {
            return $list;
        }
        else
        {
            # Retrieve the class type of the target.
            $class = $list->first()->pluck('setter_type')[0];
            # Get the list of IDs.
            $ids = $list->pluck('setter_id')->toArray();
            # Use the whereIn method of the class to return a collection of class objects.
            return $class::whereIn('id', $ids)->get();
        }
    }

    /**
     * Retrieve the full set of objects involved in a bidirectional interaction.
     *
     * @param $interaction_type
     * @return Collection
     */
    public function either($interaction_type) : Collection
    {
        $mine = $this->my($interaction_type);
        $theirs = $this->their($interaction_type);
        return $mine->concat($theirs);
    }

}
