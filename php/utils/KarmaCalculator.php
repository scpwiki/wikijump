<?php
/**
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 * 
 * @category Wikidot
 * @package Wikidot
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class KarmaCalculator {
    
    protected $_rules = array();
    
    
    public function __construct(){
        /* Init rules. */
        $rulesPath  = WIKIDOT_ROOT.'/php/utils/karmarules/';
        $files = ls($rulesPath, '*.php');
        foreach($files as $f){
            require_once($rulesPath.'/'.$f);
            $cn = str_replace('.php', '', basename($f));
            $this->_rules[] = new $cn();
        }
    }
    
    public function calculate($user){
        $p = 0;
        foreach($this->_rules as $rule) {
            $p += $rule->calculate($user);
        }
        return $p;
    }
    
    public function update($user){
        $p = $this->calculate($user);
        /* Get the karma object. */
        $c = new Criteria();
        $c->add('user_id', $user->getUserId());
        $karma = DB_UserKarmaPeer::instance()->selectOne($c);
        if(!$karma){
            $karma = new DB_UserKarma();
            $karma->setUserId($user->getUserId());
        }
        $karma->setPoints($p);
        $karma->save();
    }
    
    public function updateLevels(){
   
        /* How many points you need to have to get to a level. */
        $minPointsLevel1 = 30;
        $minPointsLevel2 = 100;
        $minPointsLevel3 = 200;
        $minPointsLevel4 = 300;
        $minPointsLevel5 = 500;
        
        /* Once you pass this limit, we will not take your level5 limit back. */
        $keepLevel5Limit = 1000;
        
        /* Calculate the distribution. */
        $db = Database::$connection;
        
        $totalUsers = DB_UserKarmaPeer::instance()->selectCount();
        /* Make karma=none for non-active users. */
        $q = "UPDATE user_karma SET level=0 WHERE points < $minPointsLevel1";
        $db->query($q);
        /* Calculate total users but excluding these with less that $minPointsLevel1 points. */
        $c = new Criteria();
        $c->add('points', $minPointsLevel1, '>=');
        $totalUsers = DB_UserKarmaPeer::instance()->selectCount($c);
        
        /* Number of users to fall into a given level. */
        $limits = array();
        $limitLevel5 = ceil($totalUsers * 0.05);
        $limitLevel4 = ceil($totalUsers * 0.10);
        $limitLevel3 = ceil($totalUsers * 0.20);
        $limitLevel2 = ceil($totalUsers * 0.30);
       
        //$c = new Criteria();
        //$c->add('points', $minPointsLevel5, '.=');
        //$c->setLimit()
        
        /* Set level one by default. */
        $q = array();
        $q[] = "UPDATE user_karma SET level=1 WHERE points >= $minPointsLevel1 AND (level < 5 OR points < $keepLevel5Limit)";
        $q[] = "UPDATE user_karma SET level=5 WHERE user_id IN (SELECT user_id FROM user_karma WHERE points >= $minPointsLevel5 ORDER BY points DESC LIMIT $limitLevel5)";
        $q[] = "UPDATE user_karma SET level=4 WHERE user_id IN (SELECT user_id FROM user_karma WHERE points >= $minPointsLevel4 AND level < 5 ORDER BY points DESC LIMIT $limitLevel4)";
        $q[] = "UPDATE user_karma SET level=3 WHERE user_id IN (SELECT user_id FROM user_karma WHERE points >= $minPointsLevel3 AND level < 4 ORDER BY points DESC LIMIT $limitLevel3)";
        $q[] = "UPDATE user_karma SET level=2 WHERE user_id IN (SELECT user_id FROM user_karma WHERE points >= $minPointsLevel2 AND level < 3 ORDER BY points DESC LIMIT $limitLevel2)";
        $db->query($q);
    }
    
    public function updateAll(){
        $offset = 0;
        $step = 1000;
        $db = Database::$connection;
        $db->begin();
        while (true) {
            $users = null;
            $c = new Criteria();
            $c->add("user_id", 0, ">");
            $c->addOrderAscending("user_id");
            $c->setLimit($step, $offset);
            $users = DB_OzoneUserPeer::instance()->select($c);
            if (count($users) == 0) {
                break;
            }
            foreach ($users as $user) {
                $this->update($user);
            }
            $offset += $step;
        }
        $this->updateLevels();
        $db->commit();
    }
    
}