<?php /* Smarty version 2.6.7, created on 2009-01-04 19:51:49
         compiled from /var/www/wikidot/templates/modules/wiki/sitestagcloud/SitesTagCloudModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('modifier', 'escape', '/var/www/wikidot/templates/modules/wiki/sitestagcloud/SitesTagCloudModule.tpl', 3, false),)), $this); ?>
<div class="sites-tag-cloud-box">
	<?php if (count($_from = (array)$this->_tpl_vars['tags'])):
    foreach ($_from as $this->_tpl_vars['tag']):
?>
		<a class="tag" href="<?php echo $this->_tpl_vars['href'];  echo ((is_array($_tmp=$this->_tpl_vars['tag']['tag'])) ? $this->_run_mod_handler('escape', true, $_tmp, 'url') : smarty_modifier_escape($_tmp, 'url')); ?>
"
			style="font-size: <?php echo $this->_tpl_vars['tag']['size']; ?>
%; color: rgb(<?php echo $this->_tpl_vars['tag']['color']['r']; ?>
, <?php echo $this->_tpl_vars['tag']['color']['g']; ?>
, <?php echo $this->_tpl_vars['tag']['color']['b']; ?>
);"
			><?php echo ((is_array($_tmp=$this->_tpl_vars['tag']['tag'])) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</a>
	<?php endforeach; endif; unset($_from); ?>
</div>