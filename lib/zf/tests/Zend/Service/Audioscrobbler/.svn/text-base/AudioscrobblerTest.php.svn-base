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
 * @package    Zend_Service_Audioscrobbler
 * @subpackage UnitTests
 * @copyright  Copyright (c) 2005-2008 Zend Technologies USA Inc. (http://www.zend.com)
 * @license    http://framework.zend.com/license/new-bsd     New BSD License
 * @version    $Id$
 */


/**
 * Test helper
 */
require_once dirname(__FILE__) . '/../../../TestHelper.php';

/**
 * @see Zend_Service_Audioscrobbler
 */
require_once 'Zend/Service/Audioscrobbler.php';


/**
 * @category   Zend
 * @package    Zend_Service_Audioscrobbler
 * @subpackage UnitTests
 * @copyright  Copyright (c) 2005-2008 Zend Technologies USA Inc. (http://www.zend.com)
 * @license    http://framework.zend.com/license/new-bsd     New BSD License
 */
class Zend_Service_Audioscrobbler_AudioscrobblerTest extends PHPUnit_Framework_TestCase
{
    public function setUp()
    {
    }
    
    public function testRequestThrowsHttpClientExceptionWithNoUserError()
    {
        $as = new Zend_Service_Audioscrobbler(true, self::readTestResponse('errorNoUserExists'));
        $as->set('user', 'foobarfoo');
        
        try {
            $response = $as->userGetProfileInformation();
            $this->fail('Expected Zend_Service_Technorati_Exception not thrown');
        } catch(Zend_Http_Client_Exception $e) {
            $this->assertContains("No user exists with this name", $e->getMessage());
        }
    }

    public function testRequestThrowsHttpClientExceptionWithoutSuccessfulResponse()
    {
        $as = new Zend_Service_Audioscrobbler(true, self::readTestResponse('errorResponseStatusError'));
        $as->set('user', 'foobarfoo');
        
        try {
            $response = $as->userGetProfileInformation();
            $this->fail('Expected Zend_Service_Technorati_Exception not thrown');
        } catch(Zend_Http_Client_Exception $e) {
            $this->assertContains("404", $e->getMessage());
        }
    }

    public static function readTestResponse($file)
    {
        return file_get_contents(dirname(__FILE__) . DIRECTORY_SEPARATOR . '_files' . DIRECTORY_SEPARATOR . $file);
    }
    

}
