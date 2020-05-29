{ 
	identifier: 'email',
	label: 'display',
	items: [

<?php
    $counter = 0;
    $ext = array("@gmail.com", "@yahoo.com", "@dojotoolkit.org");
	$aFirst = array("Adam", "Alan", "Alex", "Bill", "Becky", "Bob", "Bruce", "Carol", "Chris", "Dave", "Ed", "Ellen", "Eugene", "Frank", "Fred", "Francis", "Glenn", "James", "Jane", "Joe", "Tom"); 
	$aLast = array("Arkin", "DeBois", "Jones", "Smith", "Pitt", "Arquette", "VanDeLay");
	foreach($aFirst as $first) {
		foreach($aLast as $last) {
		    $name = $first . " " . $last;
		    $email = strtolower($first) . "." . strtolower($last) . $ext[$counter%3];
		    $display = $name . " <" . $email . ">";
			print "{ " .
					"first: '" . $first . "',\n" .
					"last: '" . $last . "',\n" .
					"email: '" . $email . "',\n" .
					"display: '" . $display . "'\n" .
				 "},\n";
			$counter ++;
		}
	}
?>
	{name: 'Blue Bell', email: 'blue.bell@gmail.com'}
   ]
}
