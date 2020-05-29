<?php

/**
 * Zend Framework
 *
 * LICENSE
 *
 * This source file is subject to the new BSD license that is bundled
 * with this package in the file LICENSE.txt.
 * It is also available through the world-wide-web at this URL:
 * http://framework.zend.com/license/new-bsd
 * If you did not receive a copy of the license and are unable to
 * obtain it through the world-wide-web, please send an email
 * to license@zend.com so we can send you a copy immediately.
 *
 * @category   Zend
 * @package    Zend_Gdata
 * @subpackage UnitTests
 * @copyright  Copyright (c) 2006 Zend Technologies USA Inc. (http://www.zend.com)
 * @license    http://framework.zend.com/license/new-bsd     New BSD License
 */

require_once dirname(dirname(dirname(__FILE__))) . DIRECTORY_SEPARATOR . 'TestHelper.php';

require_once 'Zend/Gdata/YouTube.php';
require_once 'Zend/Gdata/YouTube/VideoQuery.php';
require_once 'Zend/Gdata/ClientLogin.php';

/**
 * @package Zend_Gdata
 * @subpackage UnitTests
 */
class Zend_Gdata_YouTubeOnlineTest extends PHPUnit_Framework_TestCase
{

    public function setUp()
    {
        $this->ytAccount = constant('TESTS_ZEND_GDATA_YOUTUBE_ACCOUNT');
        $this->subscriptionTypeSchema = 'http://gdata.youtube.com/schemas/' .
            '2007/subscriptiontypes.cat';
        $this->gdata = new Zend_Gdata_YouTube();
    }

    public function tearDown()
    {
    }

    public function testRetrieveSubScriptionFeed()
    {
        $feed = $this->gdata->getSubscriptionFeed($this->ytAccount);
        $this->assertTrue($feed->totalResults->text > 0);
        $this->assertEquals('Subscriptions of zfgdata', $feed->title->text);
        $this->assertTrue(count($feed->entry) > 0);
        foreach ($feed->entry as $entry) {
            $this->assertTrue($entry->title->text != '');
        }
    }

    public function testRetrieveContactFeed()
    {
        $feed = $this->gdata->getContactFeed($this->ytAccount);
        $this->assertTrue($feed->totalResults->text > 0);
        $this->assertEquals('Contacts of zfgdata', $feed->title->text);
        $this->assertTrue(count($feed->entry) > 0);
        foreach ($feed->entry as $entry) {
            $this->assertTrue($entry->title->text != '');
        }
        $this->assertEquals('ytgdatatest1', $feed->entry[0]->username->text);
    }

    public function testRetrieveUserVideos()
    {
        $feed = $this->gdata->getUserUploads($this->ytAccount);
        $this->assertEquals('Videos uploaded by zfgdata', $feed->title->text);
        $this->assertTrue(count($feed->entry) === 1);
    }

    public function testRetrieveVideoFeed()
    {
        $feed = $this->gdata->getVideoFeed();

        $query = new Zend_Gdata_YouTube_VideoQuery();
        $query->setVideoQuery('puppy');
        $feed = $this->gdata->getVideoFeed($query);
        foreach ($feed as $videoEntry) {
            $videoResponsesLink = $videoEntry->getVideoResponsesLink();
            $videoRatingsLink = $videoEntry->getVideoRatingsLink();
            $videoComplaintsLink = $videoEntry->getVideoComplaintsLink();
        }

        $feed = $this->gdata->getVideoFeed($query->getQueryUrl());
    }

    public function testRetrieveVideoEntry()
    {
        $entry = $this->gdata->getVideoEntry('66wj2g5yz0M');
        $this->assertEquals('TestMovie', $entry->title->text);

        $entry = $this->gdata->getVideoEntry(null, 'http://gdata.youtube.com/feeds/api/videos/66wj2g5yz0M');
        $this->assertEquals('TestMovie', $entry->title->text);
    }

    public function testRetrieveOtherFeeds()
    {
        $feed = $this->gdata->getRelatedVideoFeed('66wj2g5yz0M');
        $feed = $this->gdata->getVideoResponseFeed('66wj2g5yz0M');
        $feed = $this->gdata->getVideoCommentFeed('66wj2g5yz0M');
        $feed = $this->gdata->getWatchOnMobileVideoFeed();
        $feed = $this->gdata->getUserFavorites('zfgdata');
    }

    public function testRetrieveUserProfile()
    {
        $entry = $this->gdata->getUserProfile($this->ytAccount);
        $this->assertEquals('zfgdata Channel', $entry->title->text);
        $this->assertEquals('zfgdata', $entry->username->text);
        $this->assertEquals('I\'m a lonely test account, with little to do but sit around and wait for people to use me.  I get bored in between releases and often sleep to pass the time.  Please use me more often, as I love to show off my talent in breaking your code.',
                $entry->description->text);
        $this->assertEquals(32, $entry->age->text);
        $this->assertEquals('crime and punishment, ps i love you, the stand', $entry->books->text);
        $this->assertEquals('Google', $entry->company->text);
        $this->assertEquals('software engineering, information architecture, photography, travel', $entry->hobbies->text);
        $this->assertEquals('Mountain View, CA', $entry->hometown->text);
        $this->assertEquals('San Francisco, CA 94114, US', $entry->location->text);
        $this->assertEquals('monk, heroes, law and order, top gun', $entry->movies->text);
        $this->assertEquals('imogen heap, frou frou, thievory corp, morcheeba, barenaked ladies', $entry->music->text);
        $this->assertEquals('Developer Programs', $entry->occupation->text);
        $this->assertEquals('University of the World', $entry->school->text);
        $this->assertEquals('f', $entry->gender->text);
        $this->assertEquals('taken', $entry->relationship->text);
    }

    public function testRetrieveAndUpdatePlaylistList()
    {

        $user = constant('TESTS_ZEND_GDATA_CLIENTLOGIN_EMAIL');
        $pass = constant('TESTS_ZEND_GDATA_CLIENTLOGIN_PASSWORD');
        $service = Zend_Gdata_YouTube::AUTH_SERVICE_NAME;
        $authenticationURL= 'https://www.google.com/youtube/accounts/ClientLogin';
        $httpClient = Zend_Gdata_ClientLogin::getHttpClient(
                                          $username = $user,
                                          $password = $pass,
                                          $service = $service,
                                          $client = null,
                                          $source = 'Google-UnitTests-1.0',
                                          $loginToken = null,
                                          $loginCaptcha = null,
                                          $authenticationURL);

        $this->gdata = new Zend_Gdata_YouTube($httpClient, 'Google-UnitTests-1.0', 'ytapi-gdataops-12345-u78960r7-0', 'AI39si6c-ZMGFZ5fkDAEJoCNHP9LOM2LSO1XuycZF7Eyu1IuvkioESqzRcf3voDLymIUGIrxdMx2aTufdbf5D7E51NyLYyfeaw');

        $feed = $this->gdata->getPlaylistListFeed($this->ytAccount);
        $this->assertTrue($feed->totalResults->text > 0);
        $this->assertEquals('Playlists of zfgdata', $feed->title->text);
        $this->assertTrue(count($feed->entry) > 0);
        $i = 0;
        foreach ($feed->entry as $entry) {
            $this->assertTrue($entry->title->text != '');
            if ($i == 0) {
                $entry->title->setText('new playlist title');
                $entry->save();
            }
            $i++;
        }
    }

    public function testRetrievePlaylistV2()
    {
      $this->gdata->setMajorProtocolVersion(2);
      $feed = $this->gdata->getPlaylistListFeed($this->ytAccount);
      $firstEntry = $feed->entries[0];
      $this->assertTrue($firstEntry instanceof Zend_Gdata_YouTube_PlaylistListEntry);
      $this->assertTrue($firstEntry->getSummary()->text != null);                  
    }

    public function testRetrievePlaylistVideoFeed()
    {
        $listFeed = $this->gdata->getPlaylistListFeed($this->ytAccount);

        $feed = $this->gdata->getPlaylistVideoFeed($listFeed->entry[0]->feedLink[0]->href);
        $this->assertTrue($feed->totalResults->text > 0);
        $this->assertTrue(count($feed->entry) > 0);
        foreach ($feed->entry as $entry) {
            $this->assertTrue($entry->title->text != '');
        }
    }

    public function testRetrieveTopRatedVideos()
    {
        $feed = $this->gdata->getTopRatedVideoFeed();
        $this->assertTrue($feed->totalResults->text > 10);
        $this->assertEquals('Top Rated', $feed->title->text);
        $this->assertTrue(count($feed->entry) > 0);
        foreach ($feed->entry as $entry) {
            $this->assertTrue($entry->rating->average > 3);
            $this->assertEquals(1, $entry->rating->min);
            $this->assertEquals(5, $entry->rating->max);
            $this->assertTrue($entry->rating->numRaters > 2);
        }
    }

    public function testRetrieveTopRatedVideosV2()
    {
        $this->gdata->setMajorProtocolVersion(2);
        $feed = $this->gdata->getTopRatedVideoFeed();
        $client = $this->gdata->getHttpClient();
        $positionOfAPIProjection = strpos(
            $client->getLastRequest(), "/feeds/api/");
        $this->assertTrue(is_numeric($positionOfAPIProjection));
    }

    public function testRetrieveMostViewedVideosV2()
    {
        $this->gdata->setMajorProtocolVersion(2);
        $feed = $this->gdata->getMostViewedVideoFeed();
        $client = $this->gdata->getHttpClient();
        $positionOfAPIProjection = strpos(
            $client->getLastRequest(), "/feeds/api/");
        $this->assertTrue(is_numeric($positionOfAPIProjection));
    }

    public function testRetrieveRecentlyFeaturedVideosV2()
    {
        $this->gdata->setMajorProtocolVersion(2);
        $feed = $this->gdata->getRecentlyFeaturedVideoFeed();
        $client = $this->gdata->getHttpClient();
        $positionOfAPIProjection = strpos(
            $client->getLastRequest(), "/feeds/api/");
        $this->assertTrue(is_numeric($positionOfAPIProjection));
    }

    public function testWatchOnMobileVideosV2()
    {
        $this->gdata->setMajorProtocolVersion(2);
        $feed = $this->gdata->getWatchOnMobileVideoFeed();
        $client = $this->gdata->getHttpClient();
        $positionOfAPIProjection = strpos(
            $client->getLastRequest(), "/feeds/api/");
        $this->assertTrue(is_numeric($positionOfAPIProjection));
    }

    public function testRetrieveMostViewedVideos()
    {
        $feed = $this->gdata->getMostViewedVideoFeed();
        $this->assertTrue($feed->totalResults->text > 10);
        $this->assertEquals('Most Viewed', $feed->title->text);
        $this->assertTrue(count($feed->entry) > 0);
        foreach ($feed->entry as $entry) {
            if ($entry->rating) {
                $this->assertEquals(1, $entry->rating->min);
                $this->assertEquals(5, $entry->rating->max);
            }
        }
    }
    
    public function testPerformV2Query_Location()
    {
        $this->gdata->setMajorProtocolVersion(2);
        $query = $this->gdata->newVideoQuery();
        // Setting location to New York City
        $query->setLocation('-37.0625,-95.677068');
        $query->setLocationRadius('1000km');
        $videoFeed = $this->gdata->getVideoFeed($query);
        $this->assertTrue(count($videoFeed->entry) > 0,
            'Could not retrieve a single entry for location search:' .
            $query->getQueryUrl(2));
    }
    
    public function testPerformV2Query_SafeSearch()
    {
        $this->gdata->setMajorProtocolVersion(2);
        $query = $this->gdata->newVideoQuery();
        $query->setSafeSearch('strict');
        $videoFeed = $this->gdata->getVideoFeed($query);
        $this->assertTrue(count($videoFeed->entry) > 0,
            'Could not retrieve a single entry for safeSearch=strict search:' .
            $query->getQueryUrl(2));
    }

    public function testPeformV2Query_Uploader()
    {
        $this->gdata->setMajorProtocolVersion(2);
        $query = $this->gdata->newVideoQuery();
        $query->setUploader('partner');
        $videoFeed = $this->gdata->getVideoFeed($query);
        $this->assertTrue(count($videoFeed->entry) > 0,
            'Could not retrieve a single entry for uploader=partner search:' .
            $query->getQueryUrl(2));

        foreach($videoFeed as $videoEntry) {
            $mg = $videoEntry->getMediaGroup();
            $this->assertEquals('partner',
                $mg->getMediaCredit()->getYTtype());
        }
    }
    
    public function testAddUpdateAndDeletePlaylistV2()
    {
        $user = constant('TESTS_ZEND_GDATA_CLIENTLOGIN_EMAIL');
        $pass = constant('TESTS_ZEND_GDATA_CLIENTLOGIN_PASSWORD');
        $service = Zend_Gdata_YouTube::AUTH_SERVICE_NAME;
        $authenticationURL =
            'https://www.google.com/youtube/accounts/ClientLogin';
        $httpClient = Zend_Gdata_ClientLogin::getHttpClient(
                                          $username = $user,
                                          $password = $pass,
                                          $service = $service,
                                          $client = null,
                                          $source = 'Google-UnitTests-1.0',
                                          $loginToken = null,
                                          $loginCaptcha = null,
                                          $authenticationURL);

        $this->gdata = new Zend_Gdata_YouTube(
            $httpClient, 'Google-UnitTests-1.0',
            'ytapi-gdataops-12345-u78960r7-0',
            'AI39si6c-ZMGFZ5fkDAEJoCNHP9LOM2LSO1XuycZF7E' . 
            'yu1IuvkioESqzRcf3voDLymIUGIrxdMx2aTufdbf5D7E51NyLYyfeaw');

        $this->gdata->setMajorProtocolVersion(2);
        $feed = $this->gdata->getPlaylistListFeed($this->ytAccount);

        // Add new
        $newPlaylist = $this->gdata->newPlaylistListEntry();
        $newPlaylist->setMajorProtocolVersion(2);
        $titleString = $this->generateRandomString(10);
        $newPlaylist->title = $this->gdata->newTitle()->setText($titleString);
        $newPlaylist->summary = $this->gdata->newSummary()->setText('testing');
        $postUrl = 'http://gdata.youtube.com/feeds/api/users/default/playlists';
        $successfulInsertion = true;

        try {
            $this->gdata->insertEntry($newPlaylist, $postUrl);
        } catch (Zend_Gdata_App_Exception $e) {
            $successfulInsertion = false;
        }
        
        $this->assertTrue($successfulInsertion, 'Failed to insert a new ' .
            'playlist.');

        $playlistListFeed = $this->gdata->getPlaylistListFeed('default');

        $playlistFound = false;
        $newPlaylistEntry = null;

        foreach ($playlistListFeed as $playlistListEntry) {
            if ($playlistListEntry->title->text == $titleString) {
                $playlistFound = true;
                $newPlaylistEntry = $playlistListEntry;
                break;
            }
        }
        
        $this->assertTrue($playlistFound, 'Could not find the newly inserted ' .
            'playlist.');

        // Update it
        $newTitle = $this->generateRandomString(10);
        $newPlaylistEntry->title->setText($newTitle);
        $updatedSuccesfully = true;
        try {
            $newPlaylistEntry->save();
        } catch (Zend_Gdata_App_Exception $e) {
            $updatedSuccesfully = false;
        }
        
        $this->assertTrue($updatedSuccesfully, 'Could not succesfully update ' .
            'a new playlist.');
        
        // Delete it
        $deletedSuccesfully = true;
        try {
            $newPlaylistEntry->delete();
        } catch (Zend_Gdata_App_Exception $e) {
            $deletedSuccesfully = false;
        }

        $this->assertTrue($deletedSuccesfully, 'Could not succesfully delete ' .
            'a new playlist.');
    }
    
    public function testAddAndDeleteSubscriptionToChannelV2()
    {
        $user = constant('TESTS_ZEND_GDATA_CLIENTLOGIN_EMAIL');
        $pass = constant('TESTS_ZEND_GDATA_CLIENTLOGIN_PASSWORD');
        $service = Zend_Gdata_YouTube::AUTH_SERVICE_NAME;
        $authenticationURL =
            'https://www.google.com/youtube/accounts/ClientLogin';
        $httpClient = Zend_Gdata_ClientLogin::getHttpClient(
                                          $username = $user,
                                          $password = $pass,
                                          $service = $service,
                                          $client = null,
                                          $source = 'Google-UnitTests-1.0',
                                          $loginToken = null,
                                          $loginCaptcha = null,
                                          $authenticationURL);

        $this->gdata = new Zend_Gdata_YouTube(
            $httpClient, 'Google-UnitTests-1.0',
            'ytapi-gdataops-12345-u78960r7-0',
            'AI39si6c-ZMGFZ5fkDAEJoCNHP9LOM2LSO1XuycZF7E' . 
            'yu1IuvkioESqzRcf3voDLymIUGIrxdMx2aTufdbf5D7E51NyLYyfeaw');

        $this->gdata->setMajorProtocolVersion(2);

        // Channel
        $newSubscription = $this->gdata->newSubscriptionEntry();
        $newSubscription->category = array(
            $this->gdata->newCategory('channel',
            $this->subscriptionTypeSchema));
        $newSubscription->setUsername($this->gdata->newUsername(
            'AssociatedPress'));
        
        $postUrl =
            'http://gdata.youtube.com/feeds/api/users/default/subscriptions';

        $successPosting = true;
        $message = null;
        $insertedSubscription = null;
        try {
            $insertedSubscription = $this->gdata->insertEntry(
                $newSubscription, $postUrl,
                'Zend_Gdata_YouTube_SubscriptionEntry');
        } catch (Zend_App_Exception $e) {
            $message = $e->getMessage();
            $successPosting = false;
        }
        
        $this->assertTrue($successPosting, $message);

        // Delete it
        $successDeletion = true;
        try {
            $insertedSubscription->delete();
        } catch (Zend_App_Exception $e) {
            $message = $e->getMessage();
            $successDeletion = false;
        }

        $this->assertTrue($successDeletion, $message);
    }

    public function testAddAndDeleteSubscriptionToFavoritesV2()
    {
        $user = constant('TESTS_ZEND_GDATA_CLIENTLOGIN_EMAIL');
        $pass = constant('TESTS_ZEND_GDATA_CLIENTLOGIN_PASSWORD');
        $service = Zend_Gdata_YouTube::AUTH_SERVICE_NAME;
        $authenticationURL =
            'https://www.google.com/youtube/accounts/ClientLogin';
        $httpClient = Zend_Gdata_ClientLogin::getHttpClient(
                                          $username = $user,
                                          $password = $pass,
                                          $service = $service,
                                          $client = null,
                                          $source = 'Google-UnitTests-1.0',
                                          $loginToken = null,
                                          $loginCaptcha = null,
                                          $authenticationURL);

        $this->gdata = new Zend_Gdata_YouTube(
            $httpClient, 'Google-UnitTests-1.0',
            'ytapi-gdataops-12345-u78960r7-0',
            'AI39si6c-ZMGFZ5fkDAEJoCNHP9LOM2LSO1XuycZF7E' . 
            'yu1IuvkioESqzRcf3voDLymIUGIrxdMx2aTufdbf5D7E51NyLYyfeaw');

        $this->gdata->setMajorProtocolVersion(2);

        // CBS's favorites
        $newSubscription = $this->gdata->newSubscriptionEntry();
        $newSubscription->category = array(
            $this->gdata->newCategory('favorites',
            $this->subscriptionTypeSchema));
        $newSubscription->setUsername($this->gdata->newUsername(
            'CBS'));
        
        $postUrl =
            'http://gdata.youtube.com/feeds/api/users/default/subscriptions';

        $successPosting = true;
        $message = null;
        $insertedSubscription = null;
        try {
            $insertedSubscription = $this->gdata->insertEntry(
                $newSubscription, $postUrl,
                'Zend_Gdata_YouTube_SubscriptionEntry');
        } catch (Zend_App_Exception $e) {
            $message = $e->getMessage();
            $successPosting = false;
        }
        
        $this->assertTrue($successPosting, $message);

        // Delete it
        $successDeletion = true;
        try {
            $insertedSubscription->delete();
        } catch (Zend_App_Exception $e) {
            $message = $e->getMessage();
            $successDeletion = false;
        }

        $this->assertTrue($successDeletion, $message);
    }
    
    public function testAddAndDeleteSubscriptionToPlaylistV2()
    {
        $user = constant('TESTS_ZEND_GDATA_CLIENTLOGIN_EMAIL');
        $pass = constant('TESTS_ZEND_GDATA_CLIENTLOGIN_PASSWORD');
        $service = Zend_Gdata_YouTube::AUTH_SERVICE_NAME;
        $authenticationURL =
            'https://www.google.com/youtube/accounts/ClientLogin';
        $httpClient = Zend_Gdata_ClientLogin::getHttpClient(
                                          $username = $user,
                                          $password = $pass,
                                          $service = $service,
                                          $client = null,
                                          $source = 'Google-UnitTests-1.0',
                                          $loginToken = null,
                                          $loginCaptcha = null,
                                          $authenticationURL);

        $this->gdata = new Zend_Gdata_YouTube(
            $httpClient, 'Google-UnitTests-1.0',
            'ytapi-gdataops-12345-u78960r7-0',
            'AI39si6c-ZMGFZ5fkDAEJoCNHP9LOM2LSO1XuycZF7E' . 
            'yu1IuvkioESqzRcf3voDLymIUGIrxdMx2aTufdbf5D7E51NyLYyfeaw');

        $this->gdata->setMajorProtocolVersion(2);

        // Playlist of McGyver videos
        $newSubscription = $this->gdata->newSubscriptionEntry();
        $newSubscription->setMajorProtocolVersion(2);
        $newSubscription->category = array(
            $this->gdata->newCategory('playlist',
            $this->subscriptionTypeSchema));
        $newSubscription->setPlaylistId($this->gdata->newPlaylistId(
            '7A2BB4AFFEBED2A4'));
        
        $postUrl =
            'http://gdata.youtube.com/feeds/api/users/default/subscriptions';

        $successPosting = true;
        $message = null;
        $insertedSubscription = null;
        try {
            $insertedSubscription = $this->gdata->insertEntry(
                $newSubscription, $postUrl,
                'Zend_Gdata_YouTube_SubscriptionEntry');
        } catch (Zend_App_Exception $e) {
            $message = $e->getMessage();
            $successPosting = false;
        }
        
        $this->assertTrue($successPosting, $message);

        // Delete it
        $successDeletion = true;
        try {
            $insertedSubscription->delete();
        } catch (Zend_App_Exception $e) {
            $message = $e->getMessage();
            $successDeletion = false;
        }

        $this->assertTrue($successDeletion, $message);
    }

    public function testAddAndDeleteSubscriptionToQueryV2()
    {
        $user = constant('TESTS_ZEND_GDATA_CLIENTLOGIN_EMAIL');
        $pass = constant('TESTS_ZEND_GDATA_CLIENTLOGIN_PASSWORD');
        $service = Zend_Gdata_YouTube::AUTH_SERVICE_NAME;
        $authenticationURL =
            'https://www.google.com/youtube/accounts/ClientLogin';
        $httpClient = Zend_Gdata_ClientLogin::getHttpClient(
                                          $username = $user,
                                          $password = $pass,
                                          $service = $service,
                                          $client = null,
                                          $source = 'Google-UnitTests-1.0',
                                          $loginToken = null,
                                          $loginCaptcha = null,
                                          $authenticationURL);

        $this->gdata = new Zend_Gdata_YouTube(
            $httpClient, 'Google-UnitTests-1.0',
            'ytapi-gdataops-12345-u78960r7-0',
            'AI39si6c-ZMGFZ5fkDAEJoCNHP9LOM2LSO1XuycZF7E' . 
            'yu1IuvkioESqzRcf3voDLymIUGIrxdMx2aTufdbf5D7E51NyLYyfeaw');

        $this->gdata->setMajorProtocolVersion(2);

        // Query
        $newSubscription = $this->gdata->newSubscriptionEntry();
        $newSubscription->category = array(
            $this->gdata->newCategory('query',
            $this->subscriptionTypeSchema));
        $newSubscription->setQueryString($this->gdata->newQueryString(
            'zend'));
        
        $postUrl =
            'http://gdata.youtube.com/feeds/api/users/default/subscriptions';

        $successPosting = true;
        $message = null;
        $insertedSubscription = null;
        try {
            $insertedSubscription = $this->gdata->insertEntry(
                $newSubscription, $postUrl,
                'Zend_Gdata_YouTube_SubscriptionEntry');
        } catch (Zend_App_Exception $e) {
            $message = $e->getMessage();
            $successPosting = false;
        }
        
        $this->assertTrue($successPosting, $message);

        // Delete it
        $successDeletion = true;
        try {
            $insertedSubscription->delete();
        } catch (Zend_App_Exception $e) {
            $message = $e->getMessage();
            $successDeletion = false;
        }

        $this->assertTrue($successDeletion, $message);
    }

    public function generateRandomString($length)
    {
        $outputString = null;
        for($i = 0; $i < $length; $i++) {
            $outputString .= chr(rand(65,90));
        }
        return $outputString;
    }







}
