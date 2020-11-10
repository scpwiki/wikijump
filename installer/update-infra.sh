#!/bin/bash

# Detect which init system the user is using
initsystem=unknown
if [[ `/sbin/init --version 2> /dev/null` =~ upstart ]]; then initsystem=upstart;
elif [[ `systemctl 2> /dev/null` =~ -\.mount ]]; then initsystem=systemd;
elif [[ -f /etc/init.d/cron && ! -h /etc/init.d/cron ]]; then initsystem=sysv-init;
else initsystem=unknown; fi

if [ $initsystem = unknown ]; then
  echo 'Unknown init system'
  exit 1
elif [ $initsystem = upstart ]; then
  echo 'UpStart not currently supported'
  exit 1
fi

# Update infra
if [ -z $1 ]; then
  echo "$0 (nginx|php)"
  exit 1
elif [ $1 = "nginx" ]; then
  sudo cp -r ${PWD}/etc/nginx/* /etc/nginx
  case $initsystem in
    systemd)
      sudo systemctl restart nginx
      ;;
    sysv-init)
      sudo service nginx restart
      ;;
  esac
elif [ $1 = "php" ]; then
  sudo cp ${PWD}/etc/php/7.4/cgi/conf.d/* /etc/php/7.4/cgi/conf.d
  sudo cp ${PWD}/etc/php/7.4/cgi/conf.d/* /etc/php/7.4/fpm/conf.d
  case $initsystem in
    systemd)
      sudo systemctl restart php7.4-fpm
      ;;
    sysv-init)
      sudo service php7.4-fpm restart
      ;;
  esac
else
  echo "$0 (nginx|php)"
fi
