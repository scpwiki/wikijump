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
 * @package    Zend_Mail
 * @subpackage UnitTests
 * @copyright  Copyright (c) 2005-2008 Zend Technologies USA Inc. (http://www.zend.com)
 * @license    http://framework.zend.com/license/new-bsd     New BSD License
 */

/**
 * Test helper
 */
require_once dirname(dirname(__FILE__)) . DIRECTORY_SEPARATOR . 'TestHelper.php';

/**
 * Zend_Mail
 */
require_once 'Zend/Mail.php';

/**
 * Zend_Mail_Transport_Abstract
 */
require_once 'Zend/Mail/Transport/Abstract.php';

/**
 * Zend_Mail_Transport_Sendmail
 */
require_once 'Zend/Mail/Transport/Sendmail.php';

/**
 * Zend_Mail_Transport_Smtp
 */
require_once 'Zend/Mail/Transport/Smtp.php';

/**
 * Zend_Date
 */
require_once 'Zend/Date.php';

/**
 * Mock mail transport class for testing purposes
 *
 * @category   Zend
 * @package    Zend_Mail
 * @subpackage UnitTests
 * @copyright  Copyright (c) 2005-2008 Zend Technologies USA Inc. (http://www.zend.com)
 * @license    http://framework.zend.com/license/new-bsd     New BSD License
 */
class Zend_Mail_Transport_Mock extends Zend_Mail_Transport_Abstract
{
    /**
     * @var Zend_Mail
     */
    public $mail       = null;
    public $returnPath = null;
    public $subject    = null;
    public $from       = null;
    public $headers    = null;
    public $called     = false;

    public function _sendMail()
    {
        $this->mail       = $this->_mail;
        $this->subject    = $this->_mail->getSubject();
        $this->from       = $this->_mail->getFrom();
        $this->returnPath = $this->_mail->getReturnPath();
        $this->headers    = $this->_headers;
        $this->called     = true;
    }
}

/**
 * Mock mail transport class for testing Sendmail transport
 *
 * @category   Zend
 * @package    Zend_Mail
 * @subpackage UnitTests
 * @copyright  Copyright (c) 2005-2008 Zend Technologies USA Inc. (http://www.zend.com)
 * @license    http://framework.zend.com/license/new-bsd     New BSD License
 */
class Zend_Mail_Transport_Sendmail_Mock extends Zend_Mail_Transport_Sendmail
{
    /**
     * @var Zend_Mail
     */
    public $mail    = null;
    public $from    = null;
    public $subject = null;
    public $called  = false;

    public function _sendMail()
    {
        $this->mail    = $this->_mail;
        $this->from    = $this->_mail->getFrom();
        $this->subject = $this->_mail->getSubject();
        $this->called  = true;
    }
}


/**
 * @category   Zend
 * @package    Zend_Mail
 * @subpackage UnitTests
 * @copyright  Copyright (c) 2005-2008 Zend Technologies USA Inc. (http://www.zend.com)
 * @license    http://framework.zend.com/license/new-bsd     New BSD License
 */
class Zend_MailTest extends PHPUnit_Framework_TestCase
{

    /**
     * Test case for a simple email text message with
     * multiple recipients.
     *
     */
    public function testOnlyText()
    {
        $mail = new Zend_Mail();
        $res = $mail->setBodyText('This is a test.');
        $mail->setFrom('testmail@example.com', 'test Mail User');
        $mail->setSubject('My Subject');
        $mail->addTo('recipient1@example.com');
        $mail->addTo('recipient2@example.com');
        $mail->addBcc('recipient1_bcc@example.com');
        $mail->addBcc('recipient2_bcc@example.com');
        $mail->addCc('recipient1_cc@example.com', 'Example no. 1 for cc');
        $mail->addCc('recipient2_cc@example.com', 'Example no. 2 for cc');

        $mock = new Zend_Mail_Transport_Mock();
        $mail->send($mock);

        $this->assertTrue($mock->called);
        $this->assertEquals('My Subject', $mock->subject);
        $this->assertEquals('testmail@example.com', $mock->from);
        $this->assertContains('recipient1@example.com', $mock->recipients);
        $this->assertContains('recipient2@example.com', $mock->recipients);
        $this->assertContains('recipient1_bcc@example.com', $mock->recipients);
        $this->assertContains('recipient2_bcc@example.com', $mock->recipients);
        $this->assertContains('recipient1_cc@example.com', $mock->recipients);
        $this->assertContains('recipient2_cc@example.com', $mock->recipients);
        $this->assertContains('This is a test.', $mock->body);
        $this->assertContains('Content-Transfer-Encoding: quoted-printable', $mock->header);
        $this->assertContains('Content-Type: text/plain', $mock->header);
        $this->assertContains('From: test Mail User <testmail@example.com>', $mock->header);
        $this->assertContains('Subject: My Subject', $mock->header);
        $this->assertContains('To: recipient1@example.com', $mock->header);
        $this->assertContains('Cc: Example no. 1 for cc <recipient1_cc@example.com>', $mock->header);
    }

    /**
     * Check if Header Fields are encoded correctly and if
     * header injection is prevented.
     */
    public function testHeaderEncoding()
    {
        $mail = new Zend_Mail();
        $mail->setBodyText('My Nice Test Text');
        // try header injection:
        $mail->addTo("testmail@example.com\nCc:foobar@example.com");
        $mail->addHeader('X-MyTest', "Test\nCc:foobar2@example.com", true);
        // try special Chars in Header Fields:
        $mail->setFrom('mymail@example.com', 'äüößÄÖÜ');
        $mail->addTo('testmail2@example.com', 'äüößÄÖÜ');
        $mail->addCc('testmail3@example.com', 'äüößÄÖÜ');
        $mail->setSubject('äüößÄÖÜ');
        $mail->addHeader('X-MyTest', 'Test-äüößÄÖÜ', true);

        $mock = new Zend_Mail_Transport_Mock();
        $mail->send($mock);

        $this->assertTrue($mock->called);
        $this->assertContains('From: =?iso-8859-1?Q?=E4=FC=F6=DF=C4=D6=DC?=', $mock->header);
        $this->assertNotContains("\nCc:foobar@example.com", $mock->header);
        $this->assertContains('=?iso-8859-1?Q?=E4=FC=F6=DF=C4=D6=DC?= <testmail2@example.com>', $mock->header);
        $this->assertContains('Cc: =?iso-8859-1?Q?=E4=FC=F6=DF=C4=D6=DC?= <testmail3@example.com>', $mock->header);
        $this->assertContains('Subject: =?iso-8859-1?Q?=E4=FC=F6=DF=C4=D6=DC?=', $mock->header);
        $this->assertContains('X-MyTest:', $mock->header);
        $this->assertNotContains("\nCc:foobar2@example.com", $mock->header);
        $this->assertContains('=?iso-8859-1?Q?Test-=E4=FC=F6=DF=C4=D6=DC?=', $mock->header);
    }

    /**
     * Check if Header Fields are stripped accordingly in sendmail transport;
     * also check for header injection
     * @todo Determine why this fails in Windows (testmail3@example.com example)
     */
    public function testHeaderEncoding2()
    {
        throw new PHPUnit_Framework_IncompleteTestError('still working on cross-platform tests');
        $mail = new Zend_Mail();
        $mail->setBodyText('My Nice Test Text');
        // try header injection:
        $mail->addTo("testmail@example.com\nCc:foobar@example.com");
        $mail->addHeader('X-MyTest', "Test\nCc:foobar2@example.com", true);
        // try special Chars in Header Fields:
        $mail->setFrom('mymail@example.com', 'äüößÄÖÜ');
        $mail->addTo('testmail2@example.com', 'äüößÄÖÜ');
        $mail->addCc('testmail3@example.com', 'äüößÄÖÜ');
        $mail->setSubject('äüößÄÖÜ');
        $mail->addHeader('X-MyTest', 'Test-äüößÄÖÜ', true);

        $mock = new Zend_Mail_Transport_Sendmail_Mock();
        $mail->send($mock);

        $this->assertTrue($mock->called);
        $this->assertContains('From: =?iso-8859-1?Q?"=E4=FC=F6=DF=C4=D6=DC"?=', $mock->header);
        $this->assertNotContains("\nCc:foobar@example.com", $mock->header);
        $this->assertContains('Cc: "=?iso-8859-1?Q?=E4=FC=F6=DF=C4=D6=DC?=" <testmail3@example.com>', $mock->header);
        $this->assertContains('X-MyTest:', $mock->header);
        $this->assertNotContains("\nCc:foobar2@example.com", $mock->header);
        $this->assertContains('=?iso-8859-1?Q?Test-=E4=FC=F6=DF=C4=D6=DC?=', $mock->header);

        $this->assertNotContains('Subject: ', $mock->header);
        $this->assertContains('=?iso-8859-1?Q?=E4=FC=F6=DF=C4=D6=DC?=', $mock->subject);
        $this->assertContains('"=?iso-8859-1?Q?=E4=FC=F6=DF=C4=D6=DC?=" <testmail2@example.com>', $mock->recipients, $mock->recipients);
    }

    /**
     * Check if Mails with HTML and Text Body are generated correctly.
     *
     */
    public function testMultipartAlternative()
    {
        $mail = new Zend_Mail();
        $mail->setBodyText('My Nice Test Text');
        $mail->setBodyHtml('My Nice <b>Test</b> Text');
        $mail->addTo('testmail@example.com', 'Test Recipient');
        $mail->setFrom('mymail@example.com', 'Test Sender');
        $mail->setSubject('Test: Alternate Mail with Zend_Mail');

        $mock = new Zend_Mail_Transport_Mock();
        $mail->send($mock);

        // check headers
        $this->assertTrue($mock->called);
        $this->assertContains('multipart/alternative', $mock->header);
        $boundary = $mock->boundary;
        $this->assertContains('boundary="' . $boundary . '"', $mock->header);
        $this->assertContains('MIME-Version: 1.0', $mock->header);

        // check body
        // search for first boundary
        $p1 = strpos($mock->body, "--$boundary\n");
        $this->assertNotNull($p1, $boundary . ': ' . $mock->body);

        // cut out first (Text) part
        $start1 = $p1 + 3 + strlen($boundary);
        $p2 = strpos($mock->body, "--$boundary\n", $start1);
        $this->assertNotNull($p2);

        $partBody1 = substr($mock->body, $start1, ($p2 - $start1));
        $this->assertContains('Content-Type: text/plain', $partBody1);
        $this->assertContains('My Nice Test Text', $partBody1);

        // check second (HTML) part
        // search for end boundary
        $start2 = $p2 + 3 + strlen($boundary);
        $p3 = strpos($mock->body, "--$boundary--");
        $this->assertNotNull($p3);

        $partBody2 = substr($mock->body, $start2, ($p3 - $start2));
        $this->assertContains('Content-Type: text/html', $partBody2);
        $this->assertContains('My Nice <b>Test</b> Text', $partBody2);
    }

    /**
     * check if attachment handling works
     *
     */
    public function testAttachment()
    {
        $mail = new Zend_Mail();
        $mail->setBodyText('My Nice Test Text');
        $mail->addTo('testmail@example.com', 'Test Recipient');
        $mail->setFrom('mymail@example.com', 'Test Sender');
        $mail->setSubject('Test: Attachment Test with Zend_Mail');
        $at = $mail->createAttachment('abcdefghijklmnopqrstuvexyz');
        $at->type = 'image/gif';
        $at->id = 12;
        $at->filename = 'test.gif';
        $mock = new Zend_Mail_Transport_Mock();
        $mail->send($mock);

        // now check what was generated by Zend_Mail.
        // first the mail headers:
        $this->assertContains('Content-Type: multipart/mixed', $mock->header, $mock->header);
        $boundary = $mock->boundary;
        $this->assertContains('boundary="' . $boundary . '"', $mock->header);
        $this->assertContains('MIME-Version: 1.0', $mock->header);

        // check body
        // search for first boundary
        $p1 = strpos($mock->body, "--$boundary\n");
        $this->assertNotNull($p1);

        // cut out first (Text) part
        $start1 = $p1 + 3 + strlen($boundary);
        $p2 = strpos($mock->body, "--$boundary\n", $start1);
        $this->assertNotNull($p2);

        $partBody1 = substr($mock->body, $start1, ($p2 - $start1));
        $this->assertContains('Content-Type: text/plain', $partBody1);
        $this->assertContains('My Nice Test Text', $partBody1);

        // check second (HTML) part
        // search for end boundary
        $start2 = $p2 + 3 + strlen($boundary);
        $p3 = strpos($mock->body, "--$boundary--");
        $this->assertNotNull($p3);

        $partBody2 = substr($mock->body, $start2, ($p3 - $start2));
        $this->assertContains('Content-Type: image/gif', $partBody2);
        $this->assertContains('Content-Transfer-Encoding: base64', $partBody2);
        $this->assertContains('Content-ID: <12>', $partBody2);
    }

    /**
     * Check if Mails with HTML and Text Body are generated correctly.
     *
     */
    public function testMultipartAlternativePlusAttachment()
    {
        $mail = new Zend_Mail();
        $mail->setBodyText('My Nice Test Text');
        $mail->setBodyHtml('My Nice <b>Test</b> Text');
        $mail->addTo('testmail@example.com', 'Test Recipient');
        $mail->setFrom('mymail@example.com', 'Test Sender');
        $mail->setSubject('Test: Alternate Mail with Zend_Mail');

        $at = $mail->createAttachment('abcdefghijklmnopqrstuvexyz');
        $at->type = 'image/gif';
        $at->id = 12;
        $at->filename = 'test.gif';

        $mock = new Zend_Mail_Transport_Mock();
        $mail->send($mock);

        // check headers
        $this->assertTrue($mock->called);
        $this->assertContains('multipart/mixed', $mock->header);
        $boundary = $mock->boundary;
        $this->assertContains('boundary="' . $boundary . '"', $mock->header);
        $this->assertContains('MIME-Version: 1.0', $mock->header);

        // check body
        // search for first boundary
        $p1 = strpos($mock->body, "--$boundary\n");
        $this->assertNotNull($p1);

        // cut out first (multipart/alternative) part
        $start1 = $p1 + 3 + strlen($boundary);
        $p2 = strpos($mock->body, "--$boundary\n", $start1);
        $this->assertNotNull($p2);

        $partBody1 = substr($mock->body, $start1, ($p2 - $start1));
        $this->assertContains('Content-Type: multipart/alternative', $partBody1);
        $this->assertContains('Content-Type: text/plain', $partBody1);
        $this->assertContains('Content-Type: text/html', $partBody1);
        $this->assertContains('My Nice Test Text', $partBody1);
        $this->assertContains('My Nice <b>Test</b> Text', $partBody1);

        // check second (image) part
        // search for end boundary
        $start2 = $p2 + 3 + strlen($boundary);
        $p3 = strpos($mock->body, "--$boundary--");
        $this->assertNotNull($p3);

        $partBody2 = substr($mock->body, $start2, ($p3 - $start2));
        $this->assertContains('Content-Type: image/gif', $partBody2);
        $this->assertContains('Content-Transfer-Encoding: base64', $partBody2);
        $this->assertContains('Content-ID: <12>', $partBody2);
    }

    public function testReturnPath()
    {
        $mail = new Zend_Mail();
        $res = $mail->setBodyText('This is a test.');
        $mail->setFrom('testmail@example.com', 'test Mail User');
        $mail->setSubject('My Subject');
        $mail->addTo('recipient1@example.com');
        $mail->addTo('recipient2@example.com');
        $mail->addBcc('recipient1_bcc@example.com');
        $mail->addBcc('recipient2_bcc@example.com');
        $mail->addCc('recipient1_cc@example.com', 'Example no. 1 for cc');
        $mail->addCc('recipient2_cc@example.com', 'Example no. 2 for cc');

        // First example: from and return-path should be equal
        $mock = new Zend_Mail_Transport_Mock();
        $mail->send($mock);
        $this->assertTrue($mock->called);
        $this->assertEquals($mail->getFrom(), $mock->returnPath);

        // Second example: from and return-path should not be equal
        $mail->setReturnPath('sender2@example.com');
        $mock = new Zend_Mail_Transport_Mock();
        $mail->send($mock);
        $this->assertTrue($mock->called);
        $this->assertNotEquals($mail->getFrom(), $mock->returnPath);
        $this->assertEquals($mail->getReturnPath(), $mock->returnPath);
        $this->assertNotEquals($mock->returnPath, $mock->from);
    }

    public function testNoBody()
    {
        $mail = new Zend_Mail();
        $mail->setFrom('testmail@example.com', 'test Mail User');
        $mail->setSubject('My Subject');
        $mail->addTo('recipient1@example.com');

        // First example: from and return-path should be equal
        $mock = new Zend_Mail_Transport_Mock();
        try {
            $mail->send($mock);
            $this->assertTrue($mock->called);
        } catch (Exception $e) {
            // success
            $this->assertContains('No body specified', $e->getMessage());
        }
    }

    /**
     * Helper method for {@link testZf928ToAndBccHeadersShouldNotMix()}; extracts individual header lines
     * 
     * @param Zend_Mail_Transport_Abstract $mock 
     * @param string $type 
     * @return string
     */
    protected function _getHeader(Zend_Mail_Transport_Abstract $mock, $type = 'To')
    {
        $headers = str_replace("\r\n", "\n", $mock->header);
        $headers = explode("\n", $mock->header);
        $return  = '';
        foreach ($headers as $header) {
            if (!empty($return)) {
                // Check for header continuation
                if (!preg_match('/^[a-z-]+:/i', $header)) {
                    $return .= "\r\n" . $header;
                    continue;
                } else {
                    break;
                }
            }
            if (preg_match('/^' . $type . ': /', $header)) {
                $return = $header;
            }
        }

        return $return;
    }

    public function testZf928ToAndBccHeadersShouldNotMix()
    {
        $mail = new Zend_Mail();
        $mail->setSubject('my subject');
        $mail->setBodyText('my body');
        $mail->setFrom('info@onlime.ch');
        $mail->addTo('to.address@email.com');
        $mail->addBcc('first.bcc@email.com');
        $mail->addBcc('second.bcc@email.com');

        // test with generic transport
        $mock = new Zend_Mail_Transport_Mock();
        $mail->send($mock);
        $to  = $this->_getHeader($mock);
        $bcc = $this->_getHeader($mock, 'Bcc');
        $this->assertContains('to.address@email.com', $to, $to);
        $this->assertNotContains('second.bcc@email.com', $to, $bcc);

        // test with sendmail-like transport
        $mock = new Zend_Mail_Transport_Sendmail_Mock();
        $mail->send($mock);
        $to  = $this->_getHeader($mock);
        $bcc = $this->_getHeader($mock, 'Bcc');
        // Remove the following line due to fixes by Simon
        // $this->assertNotContains('to.address@email.com', $to, $mock->header);
        $this->assertNotContains('second.bcc@email.com', $to, $bcc);
    }

    public function testZf927BlankLinesShouldPersist()
    {
        $mail = new Zend_Mail();
        $mail->setSubject('my subject');
        $mail->setBodyText("my body\r\n\r\n...after two newlines");
        $mail->setFrom('test@email.com');
        $mail->addTo('test@email.com');

        // test with generic transport
        $mock = new Zend_Mail_Transport_Sendmail_Mock();
        $mail->send($mock);
        $body = quoted_printable_decode($mock->body);
        $this->assertContains("\r\n\r\n...after", $body, $body);
    }

    public function testGetJustBodyText()
    {
        $text = "my body\r\n\r\n...after two newlines";
        $mail = new Zend_Mail();
        $mail->setBodyText($text);

        $this->assertContains('my body', $mail->getBodyText(true));
        $this->assertContains('after two newlines', $mail->getBodyText(true));
    }

    public function testGetJustBodyHtml()
    {
        $text = "<html><head></head><body><p>Some body text</p></body></html>";
        $mail = new Zend_Mail();
        $mail->setBodyHtml($text);

        $this->assertContains('Some body text', $mail->getBodyHtml(true));
    }

    public function testTypeAccessor()
    {
        $mail = new Zend_Mail();
        $this->assertNull($mail->getType());

        $mail->setType(Zend_Mime::MULTIPART_ALTERNATIVE);
        $this->assertEquals(Zend_Mime::MULTIPART_ALTERNATIVE, $mail->getType());

        $mail->setType(Zend_Mime::MULTIPART_RELATED);
        $this->assertEquals(Zend_Mime::MULTIPART_RELATED, $mail->getType());

        try {
            $mail->setType('text/plain');
            $this->fail('Invalid Zend_Mime type should throw an exception');
        } catch (Exception $e) {
        }
    }

    public function testDateSet()
    {
        $mail = new Zend_Mail();
        $res = $mail->setBodyText('Date Test');
        $mail->setFrom('testmail@example.com', 'test Mail User');
        $mail->setSubject('Date Test');
        $mail->addTo('recipient@example.com');

        $mock = new Zend_Mail_Transport_Mock();
        $mail->send($mock);

        $this->assertTrue($mock->called);
        $this->assertTrue(isset($mock->headers['Date']));
        $this->assertTrue(isset($mock->headers['Date'][0]));
        $this->assertTrue(strlen($mock->headers['Date'][0]) > 0);
    }

    public function testSetDateInt()
    {
        $mail = new Zend_Mail();
        $res = $mail->setBodyText('Date Test');
        $mail->setFrom('testmail@example.com', 'test Mail User');
        $mail->setSubject('Date Test');
        $mail->addTo('recipient@example.com');
        $mail->setDate(362656800);

        $mock = new Zend_Mail_Transport_Mock();
        $mail->send($mock);

        $this->assertTrue($mock->called);
        $this->assertTrue(strpos(implode('', $mock->headers['Date']), 'Mon, 29 Jun 1981') === 0);
    }

    public function testSetDateString()
    {
        $mail = new Zend_Mail();
        $res = $mail->setBodyText('Date Test');
        $mail->setFrom('testmail@example.com', 'test Mail User');
        $mail->setSubject('Date Test');
        $mail->addTo('recipient@example.com');
        $mail->setDate('1981-06-29T12:00:00');

        $mock = new Zend_Mail_Transport_Mock();
        $mail->send($mock);

        $this->assertTrue($mock->called);
        $this->assertTrue(strpos(implode('', $mock->headers['Date']), 'Mon, 29 Jun 1981') === 0);
    }

    public function testSetDateObject()
    {
        $mail = new Zend_Mail();
        $res = $mail->setBodyText('Date Test');
        $mail->setFrom('testmail@example.com', 'test Mail User');
        $mail->setSubject('Date Test');
        $mail->addTo('recipient@example.com');
        $mail->setDate(new Zend_Date('1981-06-29T12:00:00', Zend_Date::ISO_8601));

        $mock = new Zend_Mail_Transport_Mock();
        $mail->send($mock);

        $this->assertTrue($mock->called);
        $this->assertTrue(strpos(implode('', $mock->headers['Date']), 'Mon, 29 Jun 1981') === 0);
    }

    public function testSetDateInvalidString()
    {
        $mail = new Zend_Mail();
        
        try {
            $mail->setDate('invalid date');
            $this->fail('Invalid date should throw an exception');
        } catch (Exception $e) {
        }
    }

    public function testSetDateInvalidType()
    {
        $mail = new Zend_Mail();

        try {
            $mail->setDate(true);
            $this->fail('Invalid date should throw an exception');
        } catch (Exception $e) {
        }
    }

    public function testSetDateInvalidObject()
    {
        $mail = new Zend_Mail();

        try {
            $mail->setDate($mail);
            $this->fail('Invalid date should throw an exception');
        } catch (Exception $e) {
        }
    }

    public function testSetDateTwice()
    {
        $mail = new Zend_Mail();

        $mail->setDate();
        try {
            $mail->setDate(123456789);
            $this->fail('setting date twice should throw an exception');
        } catch (Exception $e) {
        }
    }
}
