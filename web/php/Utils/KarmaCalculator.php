<?php

namespace Wikidot\Utils;

use Illuminate\Support\Facades\DB;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Wikijump\Helpers\LegacyTools;
use Wikijump\Models\User;

class KarmaCalculator
{

    protected $_rules = [];

    public int $minPointsLevel1 = 30;
    public int $minPointsLevel2 = 100;
    public int $minPointsLevel3 = 200;
    public int $minPointsLevel4 = 300;
    public int $minPointsLevel5 = 500;


    public function __construct()
    {
        /* Init rules. */
        $rulesPath  = WIKIJUMP_ROOT.'/php/Utils/KarmaRules/';
        $files = ls($rulesPath, '*.php');
        foreach ($files as $f) {
            require_once($rulesPath.'/'.$f);
            $class = LegacyTools::getNamespacedClassFromPath($rulesPath.'/'.$f);
            $this->_rules[] = new $class();
        }
    }

    public function calculate($user)
    {
        $p = 0;
        foreach ($this->_rules as $rule) {
            $p += $rule->calculate($user);
        }
        return $p;
    }

    /**
     * Calculate karma level.
     *
     * Note that the behavior of a switch in PHP is to execute every command
     * after the first true condition without a `break` statement. If it
     * were set so if $p >= $minPointsLevelX, everyone would end up with karma
     * level 5.
     *
     * @noinspection PhpMissingBreakStatementInspection
     */
    public function update($user)
    {
        $p = $this->calculate($user);
        $user->karma_points = $p;

        switch($p) {
            case ($p < $this->minPointsLevel1):
                $user->karma_level = 0;
                /* FALLTHROUGH */
            case ($p < $this->minPointsLevel2):
                $user->karma_level = 1;
                /* FALLTHROUGH */
            case ($p < $this->minPointsLevel3):
                $user->karma_level = 2;
                /* FALLTHROUGH */
            case ($p < $this->minPointsLevel4):
                $user->karma_level = 3;
                /* FALLTHROUGH */
            case ($p < $this->minPointsLevel5):
                $user->karma_level = 4;
                break;
            default:
                $user->karma_level = 5;
        }

        $user->save();
    }

    public function updateLevels()
    {
        /**
         * This used to calculate the karma level of a user by comparing their
         * karma points to the rest of the userbase.
         * The Wikidot formula is as follows:
         * Everyone qualifies for Karma Level 1 with at least 30 points.
         * The top 30% qualify for level 2 with at least 100 points.
         * The top 20% qualify for level 3 with at least 200 points.
         * The top 10% qualify for level 4 with at least 300 points.
         * The top 5% qualify for level 5 with at least 500 points.
         * A user with 1000 or more karma points holds level 5 permanently.
         */
    }

    public function updateAll()
    {
        $db = Database::$connection;
        $db->begin();
        DB::table('users')->orderBy('id')->chunk(1000, function($users) {
           foreach ($users as $user) {
               $this->update($user);
           }
        });

        $this->updateLevels();
        $db->commit();
    }
}
