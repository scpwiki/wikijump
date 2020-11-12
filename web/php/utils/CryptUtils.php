<?php
class CryptUtils
{
    private static $keyFile = 'conf/ssl/key.pem';
    private static $publicFile = 'conf/ssl/public.pem';

    public static function generateSeed($length = 10)
    {
        $vals = '0123456789abcdefghijklmnopqrstuvwxyz';
        $charMin = 48;
        $charMax = 90;
        $out = '';
        for ($i = 0; $i<$length; $i++) {
            $out .= $vals[rand(0, 35)];
        }
        return $out;
    }

    public static function rsaGenerateModulus()
    {
        $keyFile = WIKIJUMP_ROOT.'/'.self::$keyFile;
        $keyFile = escapeshellarg($keyFile);
        $cmd = 'openssl rsa -in '.$keyFile.' -noout -modulus';
        $modulus =  exec($cmd);
        $modulus = trim(str_replace("Modulus=", '', $modulus));
        return $modulus;
    }

    public static function rsaDecrypt($text)
    {
        $keyFile = WIKIJUMP_ROOT.'/'.self::$keyFile;
        $keyFile = escapeshellarg($keyFile);
        $cmd = 'openssl base64 -d | openssl rsautl -inkey '.$keyFile.' -decrypt';
        $descriptorspec = array(
            0 => array("pipe", "r"),  // stdin is a pipe that the child will read from
            1 => array("pipe", "w"),  // stdout is a pipe that the child will write to
            2 => array("pipe", "w") // stderr is a file to write to
        );
        $process = proc_open($cmd, $descriptorspec, $pipes);
        if (!is_resource($process)) {
            throw new Exception(_("RSA decryption failed."));
        }
        fwrite($pipes[0], $text);
        fclose($pipes[0]);

        $error = stream_get_contents($pipes[2]);
        if ($error !== null && $error !== '') {
            throw new Exception(_("RSA decryption failed").": ".$error);
        }

        $result =  stream_get_contents($pipes[1]);
        fclose($pipes[1]);
        return $result;
    }

    public static function rsaEncrypt($text)
    {
        $keyFile = WIKIJUMP_ROOT.'/'.self::$publicFile;
        $keyFile = escapeshellarg($keyFile);
        $cmd = 'openssl rsautl -pubin -inkey '.$keyFile.' -encrypt | openssl base64 -e';
        $descriptorspec = array(
            0 => array("pipe", "r"),  // stdin is a pipe that the child will read from
            1 => array("pipe", "w"),  // stdout is a pipe that the child will write to
            2 => array("pipe", "w") // stderr is a file to write to
        );
        $process = proc_open($cmd, $descriptorspec, $pipes);
        if (!is_resource($process)) {
            throw new Exception(_("RSA encryption failed."));
        }
        fwrite($pipes[0], $text);
        fclose($pipes[0]);

        $error = stream_get_contents($pipes[2]);
        if ($error !== null && $error !== '') {
            throw new Exception(_("RSA decryption failed").": ".$error);
        }

        $result =  stream_get_contents($pipes[1]);
        fclose($pipes[1]);
        return $result;
    }

    public static function modulus()
    {
        $m = file_get_contents(WIKIJUMP_ROOT.'/conf/ssl/modulus.pem');
        return trim($m);
    }
}
