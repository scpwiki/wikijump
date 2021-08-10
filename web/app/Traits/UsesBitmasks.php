<?php
declare(strict_types=1);

namespace Wikijump\Traits;

/**
 * Trait UsesBitmasks, expects an int `flags` property/field on the model.
 * @package Wikijump\Traits
 */
trait UsesBitmasks
{
    /**
     * Get the current state of a flag.
     * @param int $flag
     * @return bool
     */
    public function getFlag(int $flag) : bool
    {
        return ($this->flags & $flag) === $flag;
    }

    /**
     * Set a flag to true/1.
     * @param int $flag
     * @return bool
     */
    public function setFlag(int $flag) : bool
    {
        try {
            $this->flags |= $flag;
            $this->save();
            return true;
        }
        catch (\Exception $e) {
            return false;
        }
    }

    /**
     * Set a flag to false/0.
     * @param int $flag
     * @return bool
     */
    public function clearFlag(int $flag) : bool
    {
        try {
            $this->flags &= ~$flag;
            $this->save();
            return true;
        }
        catch (\Exception $e) {
            return false;
        }
    }
}
