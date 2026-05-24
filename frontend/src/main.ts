import { route, startRouter } from './router';
import { projectList } from './components/project-list';
import { taskBoard } from './components/task-board';
import { taskDetail } from './components/task-detail';

route('/', () => projectList());
route('/projects/:id', (params) => taskBoard(params));
route('/tasks/:id', (params) => taskDetail(params));

startRouter();
