<?php

namespace Wikidot\Utils\KarmaRules;

use Wikidot\Utils\KarmaRuleInterface;

class NumberContacts implements KarmaRuleInterface
{

    public function calculate($user)
    {
        return $user->contacts()->count();
    }
}
