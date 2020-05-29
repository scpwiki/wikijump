<?php /* Smarty version 2.6.7, created on 2009-01-04 19:47:39
         compiled from /var/www/wikidot/templates/modules/wiki/sitesactivity/SomeGlobalStatsModule.tpl */ ?>
<div class="some-global-stats-box">
	<p>
		all users: <?php echo $this->_tpl_vars['totalUsers']; ?>
<br/>
		all sites: <?php echo $this->_tpl_vars['totalSites']; ?>
<br/>
		all pages: <?php echo $this->_tpl_vars['totalPages']; ?>
<br/>
		new users last 24h: <?php echo $this->_tpl_vars['newUsers']; ?>
<br/>
		page edits last 24h: <?php echo $this->_tpl_vars['recentEdits']; ?>

	</p>
</div>