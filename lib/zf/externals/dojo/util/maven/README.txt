
Maven build for Dojo Toolkit

maven                - The parent pom module with licenses, versions etc.
maven/dojo           - calls out to shrinksafe to build the standard dojo release
                       bundles as zip, tar.gz and tar.bz2
maven/java           - The parent pom for java dojo utilities
maven/java/dojo      - Packages dojo as a java Webapp.


Maven projects will be able to use the zip, tar and/or war 
maven artifacts as dependencies and include them in their
webapps as resources or templates.

Currently there is no public dojo repository. So artifacts 
are only installed to your local maven repository. So maven 
projects will need build this locally.  Eventually the artifacts 
will be published.

The artifacts are also available at:
    ./dojo/target/dojo-1.1-SNAPSHOT.tar.gz
    ./dojo/target/dojo-1.1-SNAPSHOT.tar.bz2
    ./dojo/target/dojo-1.1-SNAPSHOT.zip
    ./java/dojo/target/dojo-1.1-SNAPSHOT.war




To build everything:
  mvn 


To re build the war file

  mvn -f java/dojo/pom.xml

or

  cd java/dojo
  mvn




To run a jetty server with the war
 
  mvn -f java/dojo/pom.xml jetty:run-war

or

  cd java/dojo
  mvn jetty:run-war

Dojo is then available on port 8080













