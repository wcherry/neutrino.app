// Design tokens (CSS, not re-exported as JS)
// Import the CSS entry from 'packages/ui/src/styles/index.css'

// Primitives
export { Button } from './components/primitives/Button';
export type { ButtonProps, ButtonVariant, ButtonSize } from './components/primitives/Button';

export { Text } from './components/primitives/Text';
export type { TextProps, TextSize, TextWeight, TextColor, TextLeading, TextAs } from './components/primitives/Text';

export { Heading } from './components/primitives/Heading';
export type { HeadingProps, HeadingLevel, HeadingSize, HeadingColor, HeadingWeight } from './components/primitives/Heading';

export { Link } from './components/primitives/Link';
export type { LinkProps, LinkVariant } from './components/primitives/Link';

export { Divider } from './components/primitives/Divider';
export type { DividerProps, DividerOrientation, DividerSpacing } from './components/primitives/Divider';

export { Badge } from './components/primitives/Badge';
export type { BadgeProps, BadgeVariant, BadgeSize } from './components/primitives/Badge';

export { Avatar } from './components/primitives/Avatar';
export type { AvatarProps, AvatarSize, AvatarStatus } from './components/primitives/Avatar';

// Inputs
export { TextInput } from './components/inputs/TextInput';
export type { TextInputProps, TextInputSize } from './components/inputs/TextInput';

export { Textarea } from './components/inputs/Textarea';
export type { TextareaProps } from './components/inputs/Textarea';

export { Select } from './components/inputs/Select';
export type { SelectProps, SelectOption, SelectSize } from './components/inputs/Select';

export { Checkbox } from './components/inputs/Checkbox';
export type { CheckboxProps } from './components/inputs/Checkbox';

export { Radio, RadioGroup } from './components/inputs/Radio';
export type { RadioProps, RadioGroupProps } from './components/inputs/Radio';

export { Toggle } from './components/inputs/Toggle';
export type { ToggleProps, ToggleSize } from './components/inputs/Toggle';

export { SearchInput } from './components/inputs/SearchInput';
export type { SearchInputProps, SearchInputSize, SearchInputVariant } from './components/inputs/SearchInput';

// Feedback
export { Alert } from './components/feedback/Alert';
export type { AlertProps, AlertVariant } from './components/feedback/Alert';

export { Toast } from './components/feedback/Toast';
export type { ToastProps, ToastData, ToastVariant } from './components/feedback/Toast';

export { ToastProvider, useToast } from './components/feedback/ToastProvider';
export type { ToastProviderProps, ToastPosition } from './components/feedback/ToastProvider';

export { ProgressBar } from './components/feedback/ProgressBar';
export type { ProgressBarProps, ProgressBarSize, ProgressBarColor } from './components/feedback/ProgressBar';

export { Spinner } from './components/feedback/Spinner';
export type { SpinnerProps, SpinnerSize, SpinnerColor } from './components/feedback/Spinner';

export { Skeleton, FileListSkeleton } from './components/feedback/SkeletonLoader';
export type { SkeletonProps, SkeletonShape, FileListSkeletonProps } from './components/feedback/SkeletonLoader';

export { EmptyState } from './components/feedback/EmptyState';
export type { EmptyStateProps, EmptyStateSize } from './components/feedback/EmptyState';

// Containers
export { Card, CardHeader, CardFooter } from './components/containers/Card';
export type {
  CardProps,
  CardPadding,
  CardShadow,
  CardHeaderProps,
  CardFooterProps,
} from './components/containers/Card';

export { Panel } from './components/containers/Panel';
export type { PanelProps, PanelColor, PanelPadding } from './components/containers/Panel';

export { Modal, ModalHeader, ModalBody, ModalFooter } from './components/containers/Modal';
export type {
  ModalProps,
  ModalSize,
  ModalHeaderProps,
  ModalBodyProps,
  ModalFooterProps,
} from './components/containers/Modal';

export { Popover } from './components/containers/Popover';
export type { PopoverProps, PopoverPlacement } from './components/containers/Popover';

export { Drawer } from './components/containers/Drawer';
export type { DrawerProps, DrawerPlacement } from './components/containers/Drawer';

export { Tabs, TabList, Tab, TabPanel } from './components/containers/Tabs';
export type {
  TabsProps,
  TabListProps,
  TabProps,
  TabPanelProps,
  TabsVariant,
} from './components/containers/Tabs';

export { Accordion, AccordionItem } from './components/containers/Accordion';
export type { AccordionProps, AccordionItemProps } from './components/containers/Accordion';

// Navigation
export { Breadcrumbs } from './components/navigation/Breadcrumbs';
export type { BreadcrumbsProps, BreadcrumbItem } from './components/navigation/Breadcrumbs';

export { Pagination } from './components/navigation/Pagination';
export type { PaginationProps } from './components/navigation/Pagination';

export { Menu, MenuItem, MenuSeparator, MenuGroup } from './components/navigation/Menu';
export type {
  MenuProps,
  MenuItemProps,
  MenuItemDef,
  MenuGroupProps,
} from './components/navigation/Menu';

export { Dropdown } from './components/navigation/Dropdown';
export type { DropdownProps, DropdownPlacement } from './components/navigation/Dropdown';

// Shell
export { AppShell, useShell } from './shell/AppShell';
export type { AppShellProps } from './shell/AppShell';

export { Sidebar } from './shell/Sidebar';
export type { SidebarProps, NavItem, NavSection, StorageQuota } from './shell/Sidebar';

export { Topbar } from './shell/Topbar';
export type { TopbarProps, TopbarUser, TopbarAction } from './shell/Topbar';

// Icons
export { Icon } from './icons/Icon';
export type { IconProps } from './icons/Icon';

// Motion
export {
  fadeIn,
  slideUp,
  slideDown,
  slideLeft,
  slideRight,
  scaleIn,
  scaleUp,
  drawerLeft,
  drawerRight,
  drawerBottom,
  staggerContainer,
  staggerItem,
  toastVariants,
} from './motion/variants';
