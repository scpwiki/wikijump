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
 * @package    Zend_Mime
 * @subpackage UnitTests
 * @copyright  Copyright (c) 2005-2008 Zend Technologies USA Inc. (http://www.zend.com)
 * @license    http://framework.zend.com/license/new-bsd     New BSD License
 */

/**
 * Test helper
 */
require_once dirname(dirname(__FILE__)) . DIRECTORY_SEPARATOR . 'TestHelper.php';

/** Zend_Mail */
require_once 'Zend/Mail.php';

/** Zend_Mime */
require_once 'Zend/Mime.php';

/** PHPUnit test case */
require_once 'PHPUnit/Framework/TestCase.php';

/** Zend_MailTest */
require_once dirname(__FILE__) . '/MailTest.php';

/**
 * @category   Zend
 * @package    Zend_Mime
 * @subpackage UnitTests
 * @copyright  Copyright (c) 2005-2008 Zend Technologies USA Inc. (http://www.zend.com)
 * @license    http://framework.zend.com/license/new-bsd     New BSD License
 */
class Zend_MimeTest extends PHPUnit_Framework_TestCase
{
    public function testBoundary()
    {
        // check boundary for uniqueness
        $m1 = new Zend_Mime();
        $m2 = new Zend_Mime();
        $this->assertNotEquals($m1->boundary(), $m2->boundary());

        // check instantiating with arbitrary boundary string
        $myBoundary = 'mySpecificBoundary';
        $m3 = new Zend_Mime($myBoundary);
        $this->assertEquals($m3->boundary(), $myBoundary);

    }

    public function testIsPrintable_notPrintable()
    {
        $this->assertFalse(Zend_Mime::isPrintable('Test with special chars: �����'));
    }

	public function testIsPrintable_isPrintable()
	{
    	$this->assertTrue(Zend_Mime::isPrintable('Test without special chars'));
	}

    public function testQP()
    {
        $text = "This is a cool Test Text with special chars: ����\n"
              . "and with multiple lines���� some of the Lines are long, long"
              . ", long, long, long, long, long, long, long, long, long, long"
              . ", long, long, long, long, long, long, long, long, long, long"
              . ", long, long, long, long, long, long, long, long, long, long"
              . ", long, long, long, long and with ����";

        $qp = Zend_Mime::encodeQuotedPrintable($text);
        $this->assertEquals(quoted_printable_decode($qp), $text);
    }

    public function testBase64()
    {
        $content = str_repeat("\x88\xAA\xAF\xBF\x29\x88\xAA\xAF\xBF\x29\x88\xAA\xAF", 4);
        $encoded = Zend_Mime::encodeBase64($content);
        $this->assertEquals($content, base64_decode($encoded));
    }

    public function testZf1058WhitespaceAtEndOfBodyCausesInfiniteLoop()
    {
        $mail = new Zend_Mail();
        $mail->setSubject('my subject');
        $mail->setBodyText("my body\r\n\r\n...after two newlines\r\n ");
        $mail->setFrom('test@email.com');
        $mail->addTo('test@email.com');

        // test with generic transport
        $mock = new Zend_Mail_Transport_Sendmail_Mock();
        $mail->send($mock);
        $body = quoted_printable_decode($mock->body);
        $this->assertContains("my body\r\n\r\n...after two newlines", $body, $body);
    }
}
