<?php

namespace Tests\Legacy\Regression;

use Wikidot\DB\OzoneUser;
use Wikidot\DB\OzoneUserPeer;
use Wikidot\DB\Profile;
use Wikidot\DB\ProfilePeer;
use PHPUnit\Framework\TestCase;

class UserInfoWinModuleTest extends TestCase
{

    public function testBuild()
    {
        // Currently getting a null reference exception when trying to pull a user profile.
        $userId  = 1;
        $user = OzoneUserPeer::instance()->selectByPrimaryKey($userId);

        $this->assertInstanceOf(OzoneUser::class, $user);
        $this->assertNotNull($user->getUserId());

        $profile = ProfilePeer::instance()->selectByPrimaryKey($user->getUserId());
        $this->assertNotNull($profile);
        $this->assertInstanceOf(Profile::class, $profile);

        // Test addition of pronouns functionality.
        $profile->setPronouns('they/them');
        $profile->save();
        $pronouns = $profile->getPronouns();

        $this->assertEquals('they/them', $pronouns);
    }
}
