<?php
class FileMime
{

    protected $mime = null;
    protected $contents = "";

    protected $mimeMap = array(
        "css"   => "text/css",
        "html"  => "text/html",
    );

    protected static function execFile($params, $file)
    {
        $file = escapeshellarg($file);
        exec("file $params $file", $output, $retval);
        if ($retval == 0) {
            return join("\n", $output);
        } else {
            return null;
        }
    }

    public static function mime($file)
    {
        return self::execFile("-i -b", $file);
    }

    public static function description($file)
    {
        return self::execFile("-b", $file);
    }
}
