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

class AccountContactsModule extends AccountBaseModule {

    public function build($runData) {
        
        $user = $runData->getUser();
        
        // get all contacts
        $c = new Criteria();
        $c->add("contact.user_id", $user->getUserId());
        $c->addJoin("target_user_id", "ozone_user.user_id");
        $c->addOrderAscending("ozone_user.nick_name");
        
        $contacts = DB_ContactPeer::instance()->select($c);
        
        if (true || count($contacts) > 0) {
            // get the list who contacts you back to display emails.
            // by query
            $q = "SELECT user_id FROM contact WHERE target_user_id='" . $user->getUserId() . "'";
            $db = Database::connection();
            $res = $db->query($q);
            $back = $res->fetchAll();
            
            if ($back) {
                foreach ($back as &$b) {
                    $b = $b['user_id'];
                }
                foreach ($contacts as &$contact) {
                    if (in_array($contact->getTargetUserId(), $back)) {
                        $contact->setTemp("showEmail", true);
                    }
                }
            }
            if (!$back) {
                $back = null;
            }
            $runData->contextAdd("back", $back);
            $runData->contextAdd("countBack", count($back));
        }

        $runData->contextAdd("contacts", $contacts);
        
        $maxContacts = 10;
        $runData->contextAdd("maxContacts", $maxContacts);
    
    }
}
