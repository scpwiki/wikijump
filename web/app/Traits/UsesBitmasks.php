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
     * @return void
     */
    public function setFlag(int $flag)
    {
        $this->flags |= $flag;
    }

    /**
     * Set a flag to false/0.
     * @param int $flag
     * @return void
     */
    public function clearFlag(int $flag)
    {
        $this->flags &= ~$flag;
    }

    public static function flagIsSet(int $flag) : string
    {
        return "flags::int::bit(16) & $flag::bit(16) != 0::bit(16)";
    }

    public static function flagIsNotSet(int $flag) : string
    {
        return "flags::int::bit(16) & $flag::bit(16) = 0::bit(16)";
    }
}
