<?php /* Smarty version 2.6.7, created on 2008-12-06 17:46:08
         compiled from /var/www/wikidot/templates/screens/account/AvatarUploadTarget.tpl */ ?>
<?php echo $this->_tpl_vars['page']->setLayout('Plain'); ?>


<script type="text/javascript">
	window.parent.WIKIDOT.modules.APAvatarModule.listeners.uploaded("<?php echo $this->_tpl_vars['status']; ?>
", "<?php echo $this->_tpl_vars['im48']; ?>
", "<?php echo $this->_tpl_vars['im16']; ?>
");
</script>
<?php echo $this->_tpl_vars['status']; ?>